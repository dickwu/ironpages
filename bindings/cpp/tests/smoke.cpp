#include "ironpages.hpp"

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <iterator>
#include <limits>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

static_assert(!std::is_default_constructible_v<ironpages::Pdf>);
static_assert(!std::is_copy_constructible_v<ironpages::Pdf>);
static_assert(!std::is_copy_assignable_v<ironpages::Pdf>);
static_assert(std::is_nothrow_move_constructible_v<ironpages::Pdf>);
static_assert(std::is_nothrow_move_assignable_v<ironpages::Pdf>);
static_assert(std::is_nothrow_destructible_v<ironpages::Pdf>);

static_assert(!std::is_copy_constructible_v<ironpages::Converter>);
static_assert(!std::is_copy_assignable_v<ironpages::Converter>);
static_assert(std::is_nothrow_move_constructible_v<ironpages::Converter>);
static_assert(std::is_nothrow_move_assignable_v<ironpages::Converter>);
static_assert(std::is_nothrow_destructible_v<ironpages::Converter>);
static_assert(!std::is_default_constructible_v<ironpages::PageDimensions>);
static_assert(!std::is_default_constructible_v<ironpages::PageMargins>);

namespace {

void require(bool condition, std::string_view message) {
    if (!condition) {
        std::cerr << message << '\n';
        std::exit(EXIT_FAILURE);
    }
}

std::vector<std::uint8_t> read_file(const char* path) {
    std::ifstream file(path, std::ios::binary);
    require(file.good(), "font fixture could not be opened");
    return {std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>()};
}

void require_pdf(const ironpages::Pdf& pdf) {
    require(pdf.size() > 4, "PDF output is too short");
    require(pdf.data() != nullptr, "PDF output has no readable bytes");
    require(std::string_view(reinterpret_cast<const char*>(pdf.data()), 4) == "%PDF",
            "conversion did not return a PDF");
}

void require_contains(const ironpages::Pdf& pdf,
                      std::string_view expected,
                      std::string_view message) {
    const auto* begin = pdf.data();
    const auto* end = begin + pdf.size();
    const auto found = std::search(
        begin, end, expected.begin(), expected.end(),
        [](std::uint8_t byte, char character) {
            return byte == static_cast<std::uint8_t>(character);
        });
    require(found != end, message);
}

IronpagesBytes c_text(std::string_view value) {
    return {reinterpret_cast<const std::uint8_t*>(value.data()), value.size()};
}

std::vector<std::uint8_t> render_equivalent_c_pdf(std::string_view source) {
    IronpagesConverter* converter = nullptr;
    IronpagesBuffer* pdf = nullptr;
    IronpagesError* error = nullptr;

    require(ironpages_converter_new(&converter, &error) == IRONPAGES_STATUS_OK,
            "C baseline converter allocation failed");
    require(ironpages_converter_set_page_size(
                converter, IRONPAGES_PAGE_SIZE_LETTER, &error) ==
                IRONPAGES_STATUS_OK,
            "C baseline page size failed");
    require(ironpages_converter_set_margins(
                converter, 24.0F, 24.0F, 24.0F, 24.0F, &error) ==
                IRONPAGES_STATUS_OK,
            "C baseline margins failed");
    require(ironpages_converter_set_compress(
                converter, IRONPAGES_FALSE, &error) == IRONPAGES_STATUS_OK,
            "C baseline compression failed");
    require(ironpages_converter_set_header(
                converter, c_text("Contract header"), &error) ==
                IRONPAGES_STATUS_OK,
            "C baseline header failed");
    require(ironpages_converter_set_footer(
                converter, c_text("Page {page} / {pages}"), &error) ==
                IRONPAGES_STATUS_OK,
            "C baseline footer failed");
    require(ironpages_converter_convert_html(
                converter, c_text(source), &pdf, &error) == IRONPAGES_STATUS_OK,
            "C baseline conversion failed");

    const auto* data = ironpages_buffer_data(pdf);
    const auto size = ironpages_buffer_len(pdf);
    require(data != nullptr && size > 4, "C baseline returned no PDF bytes");
    std::vector<std::uint8_t> bytes(data, data + size);
    require(ironpages_buffer_free(&pdf) == IRONPAGES_STATUS_OK,
            "C baseline PDF release failed");
    require(ironpages_converter_free(&converter) == IRONPAGES_STATUS_OK,
            "C baseline converter release failed");
    return bytes;
}

}

int main(int argc, char** argv) {
    require(argc == 3, "expected custom-font and font-pack fixture paths");
    require(ironpages::abi_version() == IRONPAGES_ABI_VERSION,
            "ABI version query disagrees with the C header");
    require(!ironpages::version().empty(), "package version is empty");

    ironpages::Converter converter;
    converter.set_page_size(ironpages::PageSize::letter)
        .set_page_size(ironpages::PageDimensions::from_points(420.0F, 595.0F))
        .set_margins(ironpages::PageMargins::uniform(24.0F))
        .set_margins(ironpages::PageMargins::from_points(36.0F, 36.0F, 36.0F,
                                                        36.0F))
        .set_header("C++ RAII")
        .set_header_html("<strong>C++ HTML header</strong>")
        .set_footer("Page {page} / {pages}")
        .set_footer_html("<em>C++ HTML footer</em>")
        .set_compression(true)
        .set_jpeg_quality(82)
        .set_auto_resize_images(true)
        .set_image_dpi(240.0F)
        .set_filter_dpi(192.0F)
        .set_mask_dpi(240.0F)
        .set_background_raster_dpi(160.0F)
        .set_occlusion_culling(false)
        .set_sanitization(true);

    const auto custom_font = read_file(argv[1]);
    converter.add_font("ParitySans", ironpages::BytesView(custom_font));
    const auto font_pack = read_file(argv[2]);
    converter.add_font_pack(ironpages::FontPackKind::emoji,
                            ironpages::BytesView(font_pack));

    auto pdf = converter.convert_html(
        "<p style=\"font-family:ParitySans\">Hello from C++ 😀</p>");
    require_pdf(pdf);
    require_contains(pdf, "/MediaBox [0 0 420 595]",
                     "custom page dimensions were not applied");
    require_contains(pdf, "ParitySans", "custom font was not embedded");
    const auto font_pack_pdf = converter.convert_html("<p>😀</p>");
    require_contains(font_pack_pdf, "NotoEmoji",
                     "emoji font pack was not embedded");

    auto moved_pdf = std::move(pdf);
    require(!pdf, "moved-from PDF still owns the native buffer");
    require_pdf(moved_pdf);

    ironpages::Converter moved_converter = std::move(converter);
    require(!converter, "moved-from converter still owns the native handle");
    require_pdf(moved_converter.convert_markdown("# Markdown"));

    bool invalid_utf8_was_typed = false;
    const char invalid_utf8[] = {static_cast<char>(0xff)};
    try {
        (void)moved_converter.convert_html(std::string_view(invalid_utf8, 1));
    } catch (const ironpages::Error& error) {
        invalid_utf8_was_typed = error.status() == ironpages::Status::invalid_utf8;
        require(!std::string_view(error.what()).empty(), "native error message is empty");
    }
    require(invalid_utf8_was_typed, "invalid UTF-8 did not produce a typed error");

    bool moved_from_was_rejected = false;
    try {
        (void)converter.convert_html("<p>closed</p>");
    } catch (const ironpages::Error& error) {
        moved_from_was_rejected = error.status() == ironpages::Status::invalid_handle;
    }
    require(moved_from_was_rejected, "moved-from converter was not rejected safely");

    bool invalid_configuration_was_typed = false;
    try {
        moved_converter.set_image_dpi(
            std::numeric_limits<float>::infinity());
    } catch (const ironpages::Error& error) {
        invalid_configuration_was_typed =
            error.status() == ironpages::Status::invalid_argument;
    }
    require(invalid_configuration_was_typed,
            "invalid configuration did not produce a typed error");

    bool invalid_page_size_was_rejected = false;
    try {
        (void)ironpages::PageDimensions::from_points(0.0F, 595.0F);
    } catch (const std::invalid_argument&) {
        invalid_page_size_was_rejected = true;
    }
    require(invalid_page_size_was_rejected,
            "invalid page dimensions formed a public value");

    const auto equivalent_source =
        std::string_view("<h1>Deterministic C and C++ contract</h1>");
    ironpages::Converter equivalent_converter;
    equivalent_converter.set_page_size(ironpages::PageSize::letter)
        .set_margins(ironpages::PageMargins::uniform(24.0F))
        .set_compression(false)
        .set_header("Contract header")
        .set_footer("Page {page} / {pages}");
    const auto equivalent_cpp_pdf =
        equivalent_converter.convert_html(equivalent_source);
    const auto equivalent_c_pdf = render_equivalent_c_pdf(equivalent_source);
    require(equivalent_c_pdf.size() == equivalent_cpp_pdf.size() &&
                std::equal(equivalent_c_pdf.begin(), equivalent_c_pdf.end(),
                           equivalent_cpp_pdf.data()),
            "equivalent C and C++ configurations produced different PDFs");

    for (int iteration = 0; iteration < 25; ++iteration) {
        ironpages::Converter cycle;
        auto cycle_pdf = cycle.convert_html("<p>ownership cycle</p>");
        require_pdf(cycle_pdf);
    }

    require_pdf(ironpages::html_to_pdf("<p>one shot</p>"));
    require_pdf(ironpages::markdown_to_pdf("# one shot"));
    return EXIT_SUCCESS;
}
