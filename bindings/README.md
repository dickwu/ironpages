# Language bindings

Ironpages exposes one rendering engine through Rust, C, C++, .NET, Java, Python,
Ruby, browser WebAssembly, and Node.js WebAssembly. Each binding keeps the
conventions of its runtime while sharing the portable converter contract below.

## Shared capabilities

Every binding (Rust, C ABI, C++, .NET, Java, Python, Ruby, browser WebAssembly,
and Node.js WebAssembly) provides:

- HTML and Markdown to PDF bytes
- A reusable, configured converter
- Page size and margins
- PDF and raster quality controls
- HTML sanitization, on by default
- Plain-text and HTML headers and footers
- Custom TrueType fonts
- Optional CJK and emoji font packs

## Runtime extras

| Runtime | Extra capabilities |
|---|---|
| Rust | Local resource boundary, direct file output, streaming writer, async API, and opt-in remote HTTP resources through the `remote` feature |
| Python | Local resource boundary and direct file output |
| Ruby | Local resource boundary and direct file output |
| CLI | Reads an HTML or Markdown file and writes a PDF file, with a local resource boundary |
| C and C++ | Stable C ABI, C++17 RAII wrapper, and CMake and pkg-config packages |
| .NET and Java | Managed packages with Linux, macOS, and Windows native assets |
| Browser and Node.js WebAssembly | Rendering inside the JavaScript runtime from bytes your application supplies |

Remote resource loading is opt-in through the Rust crate's `remote` feature, so
applications choose explicitly when rendering may use the network.

## Published artifacts

| Runtime | Package | Supported artifact contract |
|---|---|---|
| Rust | [`ironpages`](https://crates.io/crates/ironpages) | Source crate, Rust 1.88+ |
| C | [GitHub Releases](https://github.com/dickwu/ironpages/releases) | CMake metadata, Unix pkg-config files, header, and native libraries |
| C++ | [GitHub Releases](https://github.com/dickwu/ironpages/releases) | CMake targets and C++17 RAII headers with the native libraries |
| .NET | [`Ironpages`](https://www.nuget.org/packages/Ironpages) | .NET 8+ managed assembly with Linux, macOS, and Windows native assets |
| Java | [`io.github.dickwu:ironpages`](https://central.sonatype.com/artifact/io.github.dickwu/ironpages) | Java 17+ JAR with Linux, macOS, and Windows native assets |
| Python | [`ironpages`](https://pypi.org/project/ironpages/) | CPython 3.9+ ABI3 wheels for Linux, macOS, and Windows |
| Ruby | [`ironpages`](https://rubygems.org/gems/ironpages) | Ruby 3.1+ source gem and native gems for Linux, macOS, and Windows |
| Browser WebAssembly | [`ironpages`](https://www.npmjs.com/package/ironpages) | Browser ESM entry and WebAssembly binary |
| Node.js WebAssembly | [`ironpages/node`](https://www.npmjs.com/package/ironpages) | Node.js 22/24 ESM entry using the same WebAssembly binary |

All published packages use the same Ironpages version. CI builds and exercises
the C, C++, .NET, Java, Python, and Ruby artifacts before a release can publish them.
The NuGet and Maven packages are installed and rendered on every advertised
platform without Rust.
The generated npm tarball is installed, type-checked, and rendered through
Node.js 22 and 24; the browser entry remains a separate required check.

Start with the guide for your runtime:

- [Rust](https://dickwu.github.io/ironpages/get-started/rust/)
- [CLI](https://dickwu.github.io/ironpages/get-started/cli/)
- [C](c/README.md)
- [C++](https://dickwu.github.io/ironpages/get-started/cpp/)
- [.NET](https://dickwu.github.io/ironpages/get-started/dotnet/)
- [Java](https://dickwu.github.io/ironpages/get-started/java/)
- [Python](https://dickwu.github.io/ironpages/get-started/python/)
- [Ruby](https://dickwu.github.io/ironpages/get-started/ruby/)
- [Browser JavaScript](https://dickwu.github.io/ironpages/get-started/browser/)
- [Node.js](https://dickwu.github.io/ironpages/get-started/node/)
