#pragma once

#include "ironpages/converter.hpp"

#include <cstdint>
#include <string_view>

namespace ironpages {

/// Return the stable C ABI generation exposed by the linked library.
[[nodiscard]] inline std::uint32_t abi_version() noexcept {
    return ironpages_abi_version();
}

/// Return the linked Ironpages package version.
[[nodiscard]] inline std::string_view version() noexcept {
    const char* value = ironpages_version();
    return value == nullptr ? std::string_view{} : std::string_view(value);
}

/// Convert UTF-8 HTML with a default one-shot converter.
[[nodiscard]] inline Pdf html_to_pdf(std::string_view html) {
    detail::NativeResult::require_compatible_abi();
    IronpagesBuffer* pdf = nullptr;
    IronpagesError* error = nullptr;
    const auto status =
        ironpages_html_to_pdf(detail::text(html), &pdf, &error);
    return Pdf::take(status, pdf, error);
}

/// Convert UTF-8 Markdown with a default one-shot converter.
[[nodiscard]] inline Pdf markdown_to_pdf(std::string_view markdown) {
    detail::NativeResult::require_compatible_abi();
    IronpagesBuffer* pdf = nullptr;
    IronpagesError* error = nullptr;
    const auto status =
        ironpages_markdown_to_pdf(detail::text(markdown), &pdf, &error);
    return Pdf::take(status, pdf, error);
}

}
