# Language bindings

Ironpages exposes one rendering engine through Rust, C, C++, .NET, Java, Python,
Ruby, browser WebAssembly, and Node.js WebAssembly. Each binding keeps the
conventions of its runtime while sharing the portable converter contract below.

## Capability matrix

| Capability | Rust | C ABI | C++ | .NET | Java | Python | Ruby | Browser WASM | Node.js WASM |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| HTML and Markdown to PDF bytes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Reusable configured converter | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Page size and margins | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| PDF and raster quality controls | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Sanitization | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Plain-text and HTML headers/footers | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Custom TTF fonts | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Optional CJK and emoji packs | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Local resource boundary | Yes | No | No | No | No | Yes | Yes | No | No |
| Direct file output | Yes | No | No | No | No | Yes | Yes | No | No |
| Streaming writer and async API | Yes | No | No | No | No | No | No | No | No |
| Remote HTTP resources | Opt-in | No | No | No | No | No | No | No | No |

WebAssembly receives font and document bytes from the host application. It does
not read local paths. Remote resource loading remains available only through the
Rust crate's opt-in `remote` feature so language packages do not silently gain
network access.

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
