# Ironpages for Java

Ironpages renders HTML, CSS, and Markdown to PDF in-process. The Maven artifact
contains the native renderer for Linux, macOS, and Windows, so applications do
not need Rust or a browser at runtime.

## Install

Ironpages requires Java 17 or newer.

```xml
<dependency>
  <groupId>io.github.dickwu</groupId>
  <artifactId>ironpages</artifactId>
  <version>2.0.1</version>
</dependency>
```

## Convert a document

```java
import io.github.dickwu.ironpages.HtmlConverter;
import io.github.dickwu.ironpages.PageSize;
import java.nio.file.Files;
import java.nio.file.Path;

try (var converter = new HtmlConverter()
        .setPageSize(PageSize.A4)
        .setMargin(36)) {
    var pdf = converter.convertHtml("<h1>Hello from Java</h1>");
    Files.write(Path.of("document.pdf"), pdf);
}
```

`HtmlConverter` owns native memory. Close it deterministically with
try-with-resources. A converter may move between threads while idle, but calls
on the same converter must not overlap.

HTML sanitization is enabled and resource access is disabled by default.
Custom fonts and the optional CJK or emoji packs are supplied as bytes; the
binding never downloads them.

Use `setHeaderHtml` or `setFooterHtml` for sanitized images, tables, and styled
markup in the page margins. The plain setters keep their existing text contract.

## Supported runtimes

- Linux x86-64 with glibc 2.28 or newer
- Linux ARM64 with glibc 2.28 or newer
- macOS x86-64
- macOS ARM64
- Windows x86-64

The package checks the native ABI and Ironpages version before creating a
converter and reports a clear diagnostic if the native library cannot be loaded.
