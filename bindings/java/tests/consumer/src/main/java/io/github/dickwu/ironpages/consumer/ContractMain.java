package io.github.dickwu.ironpages.consumer;

import io.github.dickwu.ironpages.FontPackKind;
import io.github.dickwu.ironpages.HtmlConverter;
import io.github.dickwu.ironpages.IronpagesInfo;
import io.github.dickwu.ironpages.PageDimensions;
import io.github.dickwu.ironpages.PageMargins;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

/** Exercises only the installed Maven package, outside the source project. */
public final class ContractMain {
  private ContractMain() {}

  public static void main(String[] arguments) throws Exception {
    var expectedVersion = requiredProperty("ironpages.expectedVersion");
    require(expectedVersion.equals(IronpagesInfo.version()), "Native version mismatch.");
    require(IronpagesInfo.abiVersion() == 1, "Native ABI mismatch.");

    var customFont = Files.readAllBytes(Path.of(requiredProperty("ironpages.customFont")));
    var fontPack = Files.readAllBytes(Path.of(requiredProperty("ironpages.fontPack")));

    try (var converter = new HtmlConverter()
        .setCustomPageSize(PageDimensions.ofPoints(320, 480))
        .setMargins(PageMargins.ofPoints(12, 13, 14, 15))
        .setMargin(12)
        .setCompression(false)
        .setJpegQuality(82)
        .setAutomaticImageResize(false)
        .setImageResolution(144)
        .setFilterResolution(96)
        .setMaskResolution(144)
        .setBackgroundResolution(120)
        .setOcclusionCulling(true)
        .setSanitization(true)
        .setHeader("Package contract")
        .setFooter("Page {page} of {pages}")
        .addFont("ParitySans", customFont)
        .addFontPack(FontPackKind.CJK_JAPANESE, fontPack)) {
      var pdf = converter.convertHtml(
          "<h1 style='font-family:ParitySans'>Java package</h1><p lang='ja'>第</p>");
      require(startsWithPdf(pdf), "HTML conversion did not produce a PDF.");
      require(contains(pdf, "/MediaBox [0 0 320 480]"), "Custom page size was lost.");
      require(contains(pdf, "DroidSansFallback"), "The font pack was not embedded.");
      require(
          startsWithPdf(converter.convertMarkdown("# Packaged Markdown")),
          "Markdown conversion did not produce a PDF.");
    }

    System.out.println("Packaged Java consumer passed for Ironpages " + expectedVersion + ".");
  }

  private static String requiredProperty(String name) {
    var value = System.getProperty(name);
    if (value == null || value.isBlank()) {
      throw new IllegalStateException("Missing required system property: " + name);
    }
    return value;
  }

  private static boolean startsWithPdf(byte[] bytes) {
    return bytes.length >= 4
        && "%PDF".equals(new String(bytes, 0, 4, StandardCharsets.US_ASCII));
  }

  private static boolean contains(byte[] bytes, String expected) {
    return new String(bytes, StandardCharsets.ISO_8859_1).contains(expected);
  }

  private static void require(boolean condition, String message) {
    if (!condition) {
      throw new AssertionError(message);
    }
  }
}
