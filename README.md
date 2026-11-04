
<div align="center">

<img width="188" alt="4" src="https://github.com/user-attachments/assets/e8b569e6-e74c-4c0f-9e84-05cf37fae3ae" />

# ironpages

Fast, in-process HTML/CSS/Markdown to PDF rendering in pure Rust. No browser,
subprocess, or system dependencies.


[![docs.rs](https://docs.rs/ironpages/badge.svg)](https://docs.rs/ironpages)
[![CI](https://github.com/dickwu/ironpages/actions/workflows/ci.yml/badge.svg)](https://github.com/dickwu/ironpages/actions)
[![deps.rs](https://deps.rs/repo/github/dickwu/ironpages/status.svg)](https://deps.rs/repo/github/dickwu/ironpages)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/ironpages.svg)](https://crates.io/crates/ironpages)
[![WASM](https://img.shields.io/badge/wasm-ready-blueviolet.svg)](../../wiki/WASM-Playground)
[![Playground](https://img.shields.io/badge/try_it-playground-blueviolet.svg)](https://dickwu.github.io/ironpages/playground/)
[![Parity report](https://img.shields.io/badge/parity-report-ff69b4.svg)](https://dickwu.github.io/ironpages/parity/reports/)

**[Website](https://dickwu.github.io/ironpages/)** | **[Get Started](https://dickwu.github.io/ironpages/get-started/)** | **[Playground](https://dickwu.github.io/ironpages/playground/)** | **[HTML-to-PDF guide](https://dickwu.github.io/ironpages/guides/html-to-pdf-rust/)** | **[Parity report](https://dickwu.github.io/ironpages/parity/reports/)** | **[Wiki](../../wiki)**

</div>

<table width="100%">
  <thead>
    <tr>
      <th align="left">Runtime</th>
      <th align="left">Documentation</th>
      <th align="left">Package</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Rust</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/rust/">Get started with Rust</a></td>
      <td><a href="https://crates.io/crates/ironpages"><img alt="Crates.io" src="https://img.shields.io/crates/v/ironpages.svg"></a></td>
    </tr>
    <tr>
      <td>C</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/c/">Get started with C</a></td>
      <td><a href="https://github.com/dickwu/ironpages/releases/latest"><img alt="GitHub release" src="https://img.shields.io/github/v/release/dickwu/ironpages?label=release"></a></td>
    </tr>
    <tr>
      <td>C++</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/cpp/">Get started with C++</a></td>
      <td><a href="https://github.com/dickwu/ironpages/releases/latest"><img alt="GitHub release" src="https://img.shields.io/github/v/release/dickwu/ironpages?label=release"></a></td>
    </tr>
    <tr>
      <td>.NET</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/dotnet/">Get started with .NET</a></td>
      <td><a href="https://www.nuget.org/packages/Ironpages"><img alt="NuGet" src="https://img.shields.io/nuget/v/Ironpages.svg"></a></td>
    </tr>
    <tr>
      <td>Java</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/java/">Get started with Java</a></td>
      <td><a href="https://central.sonatype.com/artifact/io.github.dickwu/ironpages"><img alt="Maven Central" src="https://img.shields.io/maven-central/v/io.github.dickwu/ironpages.svg"></a></td>
    </tr>
    <tr>
      <td>Python</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/python/">Get started with Python</a></td>
      <td><a href="https://pypi.org/project/ironpages/"><img alt="PyPI" src="https://img.shields.io/pypi/v/ironpages.svg"></a></td>
    </tr>
    <tr>
      <td>Ruby</td>
      <td><a href="https://dickwu.github.io/ironpages/get-started/ruby/">Get started with Ruby</a></td>
      <td><a href="https://rubygems.org/gems/ironpages"><img alt="Gem" src="https://img.shields.io/gem/v/ironpages.svg"></a></td>
    </tr>
    <tr>
      <td>JavaScript</td>
      <td>
        <a href="https://dickwu.github.io/ironpages/get-started/browser/">Get started in the browser</a>
        · <a href="https://dickwu.github.io/ironpages/get-started/node/">Get started with Node.js</a>
      </td>
      <td><a href="https://www.npmjs.com/package/ironpages"><img alt="npm" src="https://img.shields.io/npm/v/ironpages.svg"></a></td>
    </tr>
  </tbody>
</table>

**ironpages** renders HTML, CSS, and Markdown to PDF inside your application.
It is a self-contained Rust engine: parsing, the CSS cascade, layout, font
shaping, SVG, math, and PDF serialization all run in-process, from one library
or one binary. The same core ships to Rust, the CLI, C, C++, .NET, Java,
Python, Ruby, Node.js, and browser WebAssembly.

## Highlights

- **One dependency to deploy.** A single library or binary renders the document; there is no browser runtime to install or manage.
- **Document-grade layout.** Block and inline flow, flexbox, grid, floats, positioning, tables, multi-column layout, `@page`, and page headers and footers.
- **Real typography.** HarfBuzz-compatible shaping (rustybuzz), font subsetting, custom TrueType fonts, bidirectional text, and optional regional CJK and emoji font packs.
- **Rich content.** Inline and external SVG, JPEG and PNG images (data URIs, local files, and remote URLs with the `remote` feature), and LaTeX math in Markdown.
- **Predictable output.** Deterministic PDF bytes, streaming output to any `std::io::Write`, and exact flow measurement for callers that paginate themselves.
- **Safe defaults.** HTML and SVG sanitization, a canonical local-file boundary, and an opt-in network policy for remote resources.
- **Every runtime.** Rust, CLI, a stable C ABI, C++17 RAII wrappers, .NET, Java 17, Python, Ruby, and WebAssembly for browsers and Node.js.

## Install

```bash
cargo add ironpages            # Rust library
cargo install ironpages        # command-line tool
```

Optional Cargo features:

| Feature | Adds |
|---------|------|
| `remote` | Fetching remote `http`/`https` resources under a configurable `NetworkPolicy` |
| `async` | Tokio-based async file conversion |
| `wasm` | The WebAssembly entry points used by the npm package |

## Quick start

```rust
let pdf = ironpages::html_to_pdf("<h1>Hello</h1><p>World</p>")?;
std::fs::write("hello.pdf", pdf)?;

let pdf = ironpages::markdown_to_pdf("# Hello\n\nWorld")?;
```

Stream straight into a file, socket, or buffer:

```rust
let file = std::fs::File::create("hello.pdf")?;
ironpages::html_to_pdf_writer("<h1>Hello</h1>", file)?;
```

Guides for every runtime live on the [website](https://dickwu.github.io/ironpages/):
[get started](https://dickwu.github.io/ironpages/get-started/),
[HTML-to-PDF rendering in Rust](https://dickwu.github.io/ironpages/guides/html-to-pdf-rust/),
and the [browser playground](https://dickwu.github.io/ironpages/playground/).

## Builder API

`HtmlConverter` configures page geometry, quality, fonts, resources, and
running headers and footers. A converter is reusable across documents.

```rust
use ironpages::{HtmlConverter, Margin, PageSize, RasterQuality};

let pdf = HtmlConverter::new()
    .page_size(PageSize::LETTER)
    .margin(Margin::uniform(54.0))
    .raster_quality(RasterQuality {
        background_dpi: 144.0,
        ..RasterQuality::default()
    })
    .header("Quarterly report")
    .footer("Page {page} of {pages}")
    .convert("<h1>Custom page</h1>")?;
```

- `.header(...)` / `.footer(...)` print plain text and substitute `{page}` and `{pages}`.
- `.header_html(...)` / `.footer_html(...)` render styled markup, images, and tables using the document's fonts, CSS, sanitizer, and resource policy. Reserve enough top or bottom margin for the fragment.
- `RasterQuality` sets the physical DPI for source images, filters, and flattened backgrounds in one place, without changing page geometry.
- `.compress(...)`, `.jpeg_quality(...)`, and `.auto_resize_images(...)` tune output size.
- `.base_path(...)` and `.resource_root(...)` grant access to local assets.

### Measuring flow heights

`HtmlConverter::measure_sentinel_tops` lays a document out through the full
conversion pipeline and returns the y-position of each sentinel marker block.
The distance between consecutive sentinels is the exact rendered height of the
content between them, which lets an application plan its own pagination with
the same wrapping, fonts, and CSS as the final PDF.

## Command line

```bash
ironpages input.html output.pdf
ironpages document.md output.pdf
ironpages --page-size letter --landscape --margin 54 input.html output.pdf
ironpages --header "Report" --footer "Page {page} of {pages}" input.html output.pdf
ironpages --header-html '<strong>Report</strong>' input.html output.pdf
ironpages --image-dpi 300 --background-raster-dpi 144 input.html output.pdf
echo '<h1>Hello</h1>' | ironpages --stdin output.pdf
```

## Features

| Area | What you get | Details |
|------|--------------|---------|
| **HTML** | 50+ elements: headings, paragraphs, lists, tables, forms, media, semantic sections, links, `<img>`, inline `<svg>` | [Layout Engine](../../wiki/Layout-Engine) |
| **CSS** | Selectors and the full cascade, custom properties, `calc()`, flexbox, grid, floats, positioning, transforms, gradients, multi-column, `@media`, `@page`, `@font-face` | [CSS Support](../../wiki/CSS-Support) |
| **Tables** | Auto and fixed layout, collapsed and separated borders, spanning cells, header rows, and page breaks between rows | [Layout Engine](../../wiki/Layout-Engine) |
| **Fonts** | Built-in Unicode fonts (Latin, Arabic, Hebrew, and more), custom TTF embedding with subsetting, system font discovery, CJK and emoji packs | [Font System](../../wiki/Font-System) |
| **Math** | LaTeX via `$...$` and `$$...$$`: fractions, roots, matrices, Greek letters, operators, delimiters, accents | [Math Engine](../../wiki/Math-Engine) |
| **SVG** | Paths, shapes, text, gradients, transforms, clip paths, `viewBox` | [Layout Engine](../../wiki/Layout-Engine) |
| **Images** | JPEG and PNG, data URIs, local files, remote URLs (`remote` feature) | [Architecture](../../wiki/Architecture) |
| **PDF** | Bookmarks from headings, link annotations, running headers and footers, compression, streaming output | [PDF Rendering](../../wiki/PDF-Rendering) |
| **Bindings** | C ABI, C++, .NET, Java, Python, Ruby, WebAssembly (browser and Node.js) | [Bindings](bindings/README.md) |

## Fonts

```rust
let pdf = HtmlConverter::new()
    .add_font("Inter", std::fs::read("Inter.ttf")?)
    .convert(r#"<p style="font-family: Inter">Shaped with HarfBuzz</p>"#)?;
```

Fonts are shaped with [rustybuzz](https://crates.io/crates/rustybuzz),
subset to the glyphs a document uses, and embedded as CIDFontType2.

Full regional CJK and emoji coverage comes from five optional font packs:
`cjk-jp`, `cjk-kr`, `cjk-sc`, `cjk-tc`, and `emoji`. Load the ones your
application needs:

```rust
use ironpages::{FontPack, FontPackKind, HtmlConverter};

let japanese = FontPack::parse(
    FontPackKind::CjkJapanese,
    std::fs::read("ironpages-font-cjk-jp.ttf")?,
)?;
let pdf = HtmlConverter::new()
    .add_font_pack(japanese)
    .convert("<p lang='ja'>日本語</p>")?;
```

The `lang` attribute selects regional glyph forms. See
[Font System](../../wiki/Font-System) and [font-packs/README.md](font-packs/README.md).

## Math

```markdown
The equation $E = mc^2$ is famous.

$$\sum_{k=1}^{n} k = \frac{n(n+1)}{2}$$
```

See [Math Engine](../../wiki/Math-Engine).

## Language bindings

Every binding wraps the same Rust engine and exposes a reusable converter with
page geometry, quality settings, sanitization, headers and footers, custom
fonts, and font packs. The [bindings guide](bindings/README.md) covers each one.

**C and C++.** Versioned static and shared libraries for Linux, macOS, and
Windows ship in GitHub Releases, with relocatable CMake targets and pkg-config
metadata. The ABI uses opaque handles, explicit ownership, and stable status
codes; the C++17 header adds move-only RAII owners. See the
[C guide](bindings/c/README.md) and [C++ guide](bindings/cpp/README.md).

**.NET**

```bash
dotnet add package Ironpages
```

```csharp
using Ironpages;

using var converter = new HtmlConverter()
    .SetPageSize(PageSize.Letter)
    .SetFooter("Page {page} of {pages}");
byte[] pdf = converter.ConvertHtml("<h1>Hello</h1>");
```

**Java 17+**

```xml
<dependency>
  <groupId>io.github.dickwu</groupId>
  <artifactId>ironpages</artifactId>
  <version>2.0.0</version>
</dependency>
```

```java
try (var converter = new HtmlConverter()) {
    byte[] pdf = converter.convertHtml("<h1>Hello</h1>");
}
```

**Python 3.9+**

```bash
pip install ironpages
```

```python
import ironpages

converter = ironpages.HtmlConverter()
converter.page_size("Letter")
converter.footer("Page {page} of {pages}")
pdf = converter.convert("<h1>Hello</h1>")
```

**Ruby 3.1+**

```bash
gem install ironpages
```

```ruby
require "ironpages"

converter = Ironpages::HtmlConverter.new
  .page_size("Letter")
  .footer("Page {page} of {pages}")
pdf = converter.convert("<h1>Hello</h1>")
```

**JavaScript (WebAssembly)**

```bash
npm install ironpages
```

```javascript
// Browser
import init, { HtmlConverter } from 'ironpages';
await init();

const converter = new HtmlConverter();
converter.pageSize('Letter');
converter.footer('Page {page} of {pages}');
const pdf = converter.htmlToPdf('<h1>Hello</h1>');
const blob = new Blob([pdf], { type: 'application/pdf' });
converter.free();
```

```javascript
// Node.js
import init, { HtmlConverter } from 'ironpages/node';
await init();

const converter = new HtmlConverter();
const pdf = converter.htmlToPdf('<h1>Hello from Node.js</h1>');
converter.free();
```

`ironpages/node` loads the WebAssembly binary bundled in the npm package.
Documents, resources, custom fonts, and font packs are passed in as bytes. See
[WASM & Playground](../../wiki/WASM-Playground).

## Security

- **Sanitized HTML by default.** Scripts, iframes, event handlers, and `javascript:` URLs are removed; SVG is sanitized and image decoding is bounded.
- **Local files inside a boundary.** Files are read only from the canonical directory granted by `.base_path(...)` or `.resource_root(...)`, with traversal and symlink escapes rejected.
- **Remote resources by policy.** With the `remote` feature, public `http`/`https` resources load while loopback, private, link-local, metadata, multicast, and reserved addresses are denied. Redirects are re-checked, DNS answers are pinned to the connection, and bodies are capped at 10 MB by default.

```rust
use ironpages::{HtmlConverter, NetworkPolicy, RemoteHost};

let cdn: RemoteHost = "cdn.example.com".parse()?;
let policy = NetworkPolicy::default()
    .with_allow_list([cdn])
    .deny_public_ips(true)
    .max_redirects(4)
    .max_body_size(2 * 1024 * 1024);

let pdf = HtmlConverter::new()
    .network_policy(policy)
    .convert("<img src='https://cdn.example.com/logo.png'>")?;
```

Sanitization, local-file access, and remote access are configured
independently. For services that render untrusted documents, pair these
controls with host-level egress rules and per-conversion time and memory
limits. See [Resource Security](../../wiki/Resource-Security).

## Performance

Criterion measures complete, in-process conversion to PDF bytes:

| Document | Median | Conversions/sec |
|----------|-------:|----------------:|
| Simple HTML (`<h1>` + `<p>`) | **0.93 ms** | 1,080 |
| Styled HTML (CSS, lists, links) | **3.5 ms** | 285 |
| Table (5 rows, styled headers) | **5.9 ms** | 170 |
| Markdown (headings, code, lists) | **7.0 ms** | 143 |
| Full report (tables, flex, progress bars) | **15.9 ms** | 63 |
| Full report + header/footer | **15.9 ms** | 63 |
| 200 CJK text-emphasis spans | **310 ms** | 3.2 |

Measured on an Apple M2 with Rust 1.94.0 (`opt-level=3`, fat LTO, one codegen
unit). Reproduce with `cargo bench --bench conversion`.

## How it works

```
HTML/Markdown → Sanitize → Parse (html5ever) → Style cascade → Layout → PDF
```

See [Architecture](../../wiki/Architecture) for the full pipeline.

## Quality and testing

- **3,200+ unit tests**, property-based tests, and six fuzz targets.
- **Visual parity corpus.** `tests/parity/` holds 1,600+ focused HTML/CSS fixtures, each compared against a committed oracle PDF at 300 DPI. Run `scripts/parity.sh`; results are published in the [parity report](https://dickwu.github.io/ironpages/parity/reports/).
- **Golden regression tests.** `cargo test --test golden` renders synthetic documents (meal plan tables, progress notes, tall images, image table rows, EXIF-rotated JPEGs) twice, requires identical bytes, and checks page count, size, and SHA-256 against `tests/golden/expected/`. After an intended change, run `UPDATE_GOLDEN=1 cargo test --test golden`; add `GOLDEN_RASTER=1` to write PNG pages to `target/tmp/golden/`.

## Migrating from ironpress

ironpages 2.0.0 starts from the ironpress 1.7.0 tree with identical rendering
output. The major version reflects the new names:

| Area | ironpress 1.x | ironpages 2.x |
| --- | --- | --- |
| Rust crate | `ironpress = "1"` | `ironpages = "2"` (or `use ironpages as ironpress;`) |
| Error type | `IronpressError` | `IronpagesError` |
| C ABI | `ironpress_*`, `IRONPRESS_*`, `ironpress.h` | `ironpages_*`, `IRONPAGES_*`, `ironpages.h` |
| Python | `pip install ironpress`, `import ironpress` | `pip install ironpages`, `import ironpages` |
| Ruby | `gem "ironpress"` | `gem "ironpages"` |
| npm / WASM | `ironpress` | `ironpages` |
| Java | `io.github.gastongouron:ironpress` | `io.github.dickwu:ironpages` |
| .NET | `Ironpress` | `Ironpages` |

In most Rust projects the migration is a dependency rename plus
`IronpressError` → `IronpagesError`. See [CHANGELOG.md](CHANGELOG.md).

## Credits and license

ironpages is a fork of [ironpress](https://github.com/gastongouron/ironpress)
by Gaston Gouron and contributors, forked at `dickwu/ironpress@db93fcdc`
(ironpress 1.7.0). The golden regression manifest is inspired by
[fullbleed](https://github.com/fullbleed-engine/fullbleed-official) (MIT) and
reimplemented here. See [NOTICE](NOTICE) and [ROADMAP.md](ROADMAP.md).

Released under the MIT license. See [LICENSE](LICENSE).
