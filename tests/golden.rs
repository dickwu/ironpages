//! Golden regression harness.
//!
//! Each `tests/golden/fixtures/*.html` file is synthetic (no real client data).
//! It is rendered with a fixed, uncompressed configuration and only the fonts
//! bundled in `assets/`, and the result is compared with the manifest for the
//! current platform, `tests/golden/expected/<os>-<arch>.txt` (page count, byte
//! length, SHA-256). Every run also renders each fixture twice and requires
//! byte-identical output.
//!
//! Manifests are per platform because floating-point and image-codec details
//! may legitimately differ between targets. A platform without a manifest only
//! gets the determinism check (set `GOLDEN_STRICT=1` to make that an error);
//! bless it by running the update command below on that platform.
//!
//! * `cargo test --test golden` verifies.
//! * `UPDATE_GOLDEN=1 cargo test --test golden` rewrites the manifest after an
//!   intentional rendering change; review the diff and the rasterized pages.
//! * `GOLDEN_RASTER=1` also writes each PDF and, when `pdftoppm` is on `PATH`,
//!   72 dpi PNG pages under `target/tmp/golden/<case>/` for visual review.
//!
//! The idea of a checked-in golden manifest of output hashes plus optional
//! rasterized pages is borrowed from fullbleed's public golden suite
//! (MIT, https://github.com/fullbleed-engine/fullbleed-official); this is an
//! independent Rust reimplementation, no fullbleed code is copied.

use ironpages::{HtmlConverter, PageSize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const MANIFEST_HEADER: &str = "# ironpages golden manifest: <case> pages=<n> bytes=<n> sha256=<hex>\n\
# Regenerate with UPDATE_GOLDEN=1 cargo test --test golden\n";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Golden {
    pages: usize,
    bytes: usize,
    sha256: String,
}

impl Golden {
    fn line(&self, case: &str) -> String {
        format!(
            "{case} pages={} bytes={} sha256={}",
            self.pages, self.bytes, self.sha256
        )
    }
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn manifest_path() -> PathBuf {
    golden_dir().join("expected").join(format!(
        "{}-{}.txt",
        std::env::consts::OS,
        std::env::consts::ARCH
    ))
}

fn fixtures() -> Vec<(String, PathBuf)> {
    let mut cases: Vec<(String, PathBuf)> = std::fs::read_dir(golden_dir().join("fixtures"))
        .expect("read golden fixtures")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? != "html" {
                return None;
            }
            Some((path.file_stem()?.to_string_lossy().into_owned(), path))
        })
        .collect();
    cases.sort();
    cases
}

fn render(path: &Path) -> Vec<u8> {
    let html = std::fs::read_to_string(path).expect("read fixture");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    HtmlConverter::new()
        .page_size(PageSize::A4)
        .compress(false)
        .base_path(path.parent().expect("fixture dir"))
        .resource_root(root)
        .convert(&html)
        .unwrap_or_else(|error| panic!("render {}: {error}", path.display()))
}

fn page_count(pdf: &[u8]) -> usize {
    let needle = b"/Type /Page";
    pdf.windows(needle.len() + 1)
        .filter(|w| w.starts_with(needle) && w[needle.len()] != b's')
        .count()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn measure(pdf: &[u8]) -> Golden {
    Golden {
        pages: page_count(pdf),
        bytes: pdf.len(),
        sha256: hex(&Sha256::digest(pdf)),
    }
}

fn parse_manifest(text: &str) -> BTreeMap<String, Golden> {
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut parts = line.split_whitespace();
            let case = parts.next().expect("case name").to_string();
            let mut field = |key: &str| {
                let part = parts
                    .next()
                    .unwrap_or_else(|| panic!("{case}: missing {key}"));
                part.strip_prefix(key)
                    .and_then(|v| v.strip_prefix('='))
                    .unwrap_or_else(|| panic!("{case}: expected {key}=, got {part}"))
                    .to_string()
            };
            let pages = field("pages").parse().expect("pages");
            let bytes = field("bytes").parse().expect("bytes");
            let sha256 = field("sha256");
            (
                case,
                Golden {
                    pages,
                    bytes,
                    sha256,
                },
            )
        })
        .collect()
}

fn write_artifacts(case: &str, pdf: &[u8]) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("golden")
        .join(case);
    std::fs::create_dir_all(&dir).expect("create artifact dir");
    let pdf_path = dir.join(format!("{case}.pdf"));
    std::fs::write(&pdf_path, pdf).expect("write artifact pdf");
    let _ = std::process::Command::new("pdftoppm")
        .args(["-r", "72", "-png"])
        .arg(&pdf_path)
        .arg(dir.join("page"))
        .status();
}

#[test]
fn golden_fixtures_match_manifest() {
    let manifest_path = manifest_path();
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let raster = std::env::var_os("GOLDEN_RASTER").is_some();
    let manifest = std::fs::read_to_string(&manifest_path).ok();
    if manifest.is_none() && !update {
        assert!(
            std::env::var_os("GOLDEN_STRICT").is_none(),
            "no golden manifest at {}",
            manifest_path.display()
        );
        eprintln!(
            "golden: no manifest at {}; checking determinism only",
            manifest_path.display()
        );
    }
    let compare = manifest.is_some();
    let expected = manifest
        .map(|text| parse_manifest(&text))
        .unwrap_or_default();

    let mut actual = BTreeMap::new();
    let mut failures = Vec::new();
    for (case, path) in fixtures() {
        let pdf = render(&path);
        assert!(pdf.starts_with(b"%PDF-"), "{case}: not a PDF");
        let again = render(&path);
        assert!(pdf == again, "{case}: rendering is not deterministic");
        let got = measure(&pdf);
        let matches = expected.get(&case) == Some(&got);
        if raster || (compare && !update && !matches) {
            write_artifacts(&case, &pdf);
        }
        match expected.get(&case) {
            _ if !compare => {}
            Some(want) if *want == got => {}
            Some(want) => failures.push(format!(
                "{case}: expected `{}`, got `{}`",
                want.line(&case),
                got.line(&case)
            )),
            None => failures.push(format!("{case}: missing from manifest")),
        }
        actual.insert(case, got);
    }
    for case in expected.keys().filter(|case| !actual.contains_key(*case)) {
        failures.push(format!("{case}: in manifest but no fixture"));
    }

    if update {
        let mut text = String::from(MANIFEST_HEADER);
        for (case, golden) in &actual {
            text.push_str(&golden.line(case));
            text.push('\n');
        }
        std::fs::create_dir_all(manifest_path.parent().expect("manifest dir"))
            .expect("create manifest dir");
        std::fs::write(&manifest_path, text).expect("write manifest");
        return;
    }
    assert!(
        failures.is_empty(),
        "golden mismatches (artifacts in {}/golden):\n{}\nIf the change is intended, rerun with UPDATE_GOLDEN=1.",
        env!("CARGO_TARGET_TMPDIR"),
        failures.join("\n")
    );
}

#[test]
fn golden_fixtures_are_synthetic() {
    // Guard against real documents being dropped into the corpus: every
    // fixture must carry an explicit synthetic marker.
    for (case, path) in fixtures() {
        let html = std::fs::read_to_string(&path).expect("read fixture");
        assert!(
            html.contains("Synthetic fixture"),
            "{case}: golden fixtures must be synthetic and say so in a comment"
        );
    }
}
