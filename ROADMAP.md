# ironpages roadmap

ironpages 2.0.0 is a rename of ironpress 1.7.0 with updated dependencies, CI and
a golden regression harness. Rendering behaviour is unchanged. This roadmap
lists the next pieces of work, roughly in priority order. Each item should land
as a small, reviewable change with a golden fixture showing the before/after
output (`UPDATE_GOLDEN=1 cargo test --test golden`, then review the diff in
`tests/golden/expected/` and the PNGs written with `GOLDEN_RASTER=1`).

Ground rules:

- No large rewrites. Prefer targeted fixes that can also be offered upstream to
  ironpress.
- Fixtures are synthetic only. Never commit real client or patient documents;
  `tests/golden.rs` refuses fixtures without a "Synthetic fixture" marker.
- Ideas credited to fullbleed (MIT) are reimplemented from its public
  documentation, not ported from its code. See NOTICE.

## 1. EXIF orientation

**Status:** bug, reproduced by `tests/golden/fixtures/exif_orientation.html`.

JPEGs carrying an EXIF `Orientation` tag (common for phone photos) are embedded
in stored pixel order, so an `Orientation=6` portrait photo appears rotated 90°.
The golden render shows the arrow pointing right instead of down.

Plan: read the orientation tag when decoding (the `image` crate exposes
`ImageDecoder::orientation()` and `DynamicImage::apply_orientation`). For
images that are re-encoded, apply the transform to the pixels. For JPEG
pass-through, either re-encode oriented images or emit the rotation/flip
in the image placement matrix and swap the intrinsic width and height for
layout. Cover orientations 1–8 with
synthetic fixtures. CSS `image-orientation: none` should keep the raw order.
fullbleed does not handle EXIF, so there is nothing to borrow here.

## 2. Tall-image clipping

**Status:** bug, reproduced by `tests/golden/fixtures/tall_image.html`.

A replaced element taller than the remaining page area is drawn on the
current page and clipped at the bottom margin. The rest of the image is lost,
and the following content starts on the next page. In the fixture only about
half of the 1800 px strip is visible.

Plan: when an image is taller than a full page content box, scale it down to
fit (Chrome's print behaviour for `max-height`-less images is to slice, but
scaling avoids losing content and matches what document users expect). An
opt-in mode could slice it instead, continuing on the next page with an offset
clip. If it fits on a fresh page, move it to the next page instead of clipping.
Add fixtures for both paths.

## 3. Table-row image slicing at page breaks

**Status:** bug, reproduced by `tests/golden/fixtures/table_row_images.html`.

When a page break falls inside a table row, the row is split and its image is
sliced across the two pages (row 6 in the fixture). Meal plans and progress
notes with thumbnails hit this case.

Plan: treat rows that contain atomic content (images, `break-inside: avoid`
cells) as unbreakable when the row fits on a fresh page, so the whole row moves
to the next page with the repeated `thead`. Keep splitting only rows taller
than a page. Respect `break-inside: avoid` on `tr` explicitly.

## 4. Memory use (~460 MB peak)

**Status:** measured. A synthetic seven-section meal plan with one 3000×2000
JPEG per section peaks at about 448 MB RSS on macOS arm64 (`/usr/bin/time -l`).
ironpress 1.7.0 and ironpages 2.0.0 measure the same; production documents
were reported at ~460 MB.

Likely contributors: decoding every image to full-resolution RGBA before
downscaling, keeping decoded images alive for the whole conversion, and
building the full PDF in memory.

Plan:

1. Add a reproducible memory benchmark using synthetic images (e.g. using
   `dhat` or a counting allocator) and record the baseline.
2. Downscale while decoding (JPEG DCT scaling) and drop decoded buffers once
   they have been encoded into the PDF.
3. Deduplicate identical images by content hash. This already exists for some
   paths; verify it is applied everywhere.
4. Consider streaming page content to the output writer.

Target: under 200 MB for the benchmark document without changing output.

## 5. Render-speed regression against 0.2.5

**Status:** reported, not yet reproduced. In the gfit application, seven meal
plans take about 30 s with the current engine against about 20 s with the
html-pdf 0.2.5 build. The synthetic memory benchmark above renders in about
1.3 s, so the slowdown depends on content that is not modelled yet. Likely
candidates are many fonts or `@font-face` rules, large tables with
`rowspan`, page-break search, or repeated stylesheet parsing per document.

Plan:

1. Build a synthetic document that mirrors the production structure (fonts,
   table shapes, image count and sizes) without any real data, and add it to
   `benches/`.
2. Profile it (`cargo flamegraph` / Instruments) on the old engine and the
   current one, and bisect.
3. Add a CI benchmark guard (criterion with a stored baseline, non-blocking at
   first).

## 6. Template precompilation and data binding

**Status:** design. Idea credited to fullbleed's compiled templates and
`{{slot}}` binding for batch/VDP output.

Applications like gfit render the same layout many times with different data.
Today every call re-parses HTML and CSS and resolves fonts.

Plan:

1. `HtmlConverter::compile(html) -> CompiledTemplate` caches the parsed
   stylesheet, font set and DOM. The first step needs no API break: cache the
   parsed CSS and loaded fonts across `convert` calls on one converter.
2. Data binding: named text slots (`{{name}}`, HTML-escaped by default) filled
   from a map, so the DOM is cloned and patched rather than re-parsed.
3. Batch API: render many records into one PDF or many PDFs, sharing font
   subsets where possible.
4. Expose it through the C ABI and the bindings once stable.

## 7. PDF/A output

**Status:** design. The profile names and requirements follow the
public PDF/A specs; fullbleed's `pdf_profile` option was the prompt.

Plan: add `PdfProfile::{PdfA2b, PdfA3b}` (and possibly `PdfA2u`) to the
converter. A profile needs:

- an embedded sRGB ICC output intent (`/OutputIntents`)
- XMP metadata with `pdfaid:part` and `pdfaid:conformance`
- every font embedded, with no unembedded base-14 font resources (the converter
  currently declares base-14 fonts even when they are unused)
- no transparency for A-1
- a document ID

Then validate the output in CI with veraPDF on synthetic fixtures.
