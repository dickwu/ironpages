# ironpages for Ruby

Generate PDF output from HTML or Markdown with a native Rust renderer. No
browser process or system PDF dependency is required.

[Read the complete Ruby getting-started guide](https://dickwu.github.io/ironpages/get-started/ruby/).

## Requirements

- Ruby 3.1 or later
- Linux, macOS, or Windows

Releases include supported native gems plus a source gem.

## Install

```bash
bundle add ironpages
```

Or install the gem globally:

```bash
gem install ironpages
```

## Create your first PDF

```ruby
require "ironpages"

pdf = Ironpages.html_to_pdf("<h1>Hello from Ruby</h1>")
File.binwrite("output.pdf", pdf)
```

The result is a binary Ruby `String`. Write it with `File.binwrite`, return it
from a web response, or store it in your application.

## Render Markdown

```ruby
pdf = Ironpages.markdown_to_pdf("# Release notes\n\nEverything shipped.")
```

## Configure and reuse a converter

```ruby
converter = Ironpages::HtmlConverter.new
  .page_size("Letter")
  .margin_sides(36, 48, 36, 48)
  .header("Quarterly report")
  .footer("Page {page} of {pages}")

# Use header_html/footer_html for sanitized images, tables, and styled markup.
converter.header_html("<strong>Quarterly report</strong>")

pdf = converter.convert("<h1>Results</h1>")
markdown_pdf = converter.convert_markdown("# Results")
```

Configuration methods return the converter, so policies compose in an idiomatic
chain. Use `convert_to_file` or `convert_markdown_to_file` for direct output.

## Local resources and fonts

Local URLs are denied until a canonical boundary is configured:

```ruby
converter = Ironpages::HtmlConverter.new
  .base_path("templates")
  .resource_root(".")
  .add_font("Inter", File.binread("assets/Inter.ttf"))
```

Optional fallback packs enter through `add_font_pack`. Valid names are
`cjk-jp`, `cjk-kr`, `cjk-sc`, `cjk-tc`, and `emoji`. Rendering never downloads
a font pack.

## Errors

- Invalid named settings raise `ArgumentError`.
- Conversion failures raise `RuntimeError`.
- HTML sanitization is enabled by default.

See the [bindings overview](../README.md) for the capabilities shared with
Rust, Python, browser WebAssembly, and Node.js.
