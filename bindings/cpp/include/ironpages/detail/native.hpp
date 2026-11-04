#pragma once

#include "ironpages/types.hpp"

#include <memory>
#include <string>
#include <string_view>
#include <utility>

namespace ironpages::detail {

struct BufferDeleter final {
    void operator()(IronpagesBuffer* buffer) const noexcept {
        if (buffer != nullptr) {
            (void)ironpages_buffer_free(&buffer);
        }
    }
};

struct ConverterDeleter final {
    void operator()(IronpagesConverter* converter) const noexcept {
        if (converter != nullptr) {
            (void)ironpages_converter_free(&converter);
        }
    }
};

struct ErrorDeleter final {
    void operator()(IronpagesError* error) const noexcept {
        if (error != nullptr) {
            (void)ironpages_error_free(&error);
        }
    }
};

using BufferOwner = std::unique_ptr<IronpagesBuffer, BufferDeleter>;
using ConverterOwner = std::unique_ptr<IronpagesConverter, ConverterDeleter>;
using ErrorOwner = std::unique_ptr<IronpagesError, ErrorDeleter>;

/// Parses native results once and releases every native error owner.
class NativeResult final {
public:
    static void require_compatible_abi() {
        const auto linked_version = ironpages_abi_version();
        if (linked_version != IRONPAGES_ABI_VERSION) {
            throw contract_violation(
                "Ironpages C++ requires ABI " +
                std::to_string(IRONPAGES_ABI_VERSION) + ", but ABI " +
                std::to_string(linked_version) + " was loaded.");
        }
    }

    static void require_success(IronpagesStatus status,
                                IronpagesError* raw_error) {
        ErrorOwner error(raw_error);
        if (status == IRONPAGES_STATUS_OK && !error) {
            return;
        }
        if (status == IRONPAGES_STATUS_OK) {
            throw contract_violation(
                "A successful native call returned an error owner.");
        }
        throw Error(category(status), status, message(error.get(), status));
    }

    [[nodiscard]] static Error contract_violation(std::string message) {
        return Error(Status::internal, IRONPAGES_STATUS_INTERNAL,
                     std::move(message));
    }

private:
    [[nodiscard]] static Status category(IronpagesStatus status) noexcept {
        switch (status) {
            case IRONPAGES_STATUS_INVALID_ARGUMENT:
                return Status::invalid_argument;
            case IRONPAGES_STATUS_INVALID_UTF8:
                return Status::invalid_utf8;
            case IRONPAGES_STATUS_INVALID_ENUM:
                return Status::invalid_enum;
            case IRONPAGES_STATUS_INVALID_HANDLE:
                return Status::invalid_handle;
            case IRONPAGES_STATUS_OUTPUT_NOT_EMPTY:
                return Status::output_not_empty;
            case IRONPAGES_STATUS_PARSE:
                return Status::parse;
            case IRONPAGES_STATUS_CSS:
                return Status::css;
            case IRONPAGES_STATUS_LAYOUT:
                return Status::layout;
            case IRONPAGES_STATUS_RENDER:
                return Status::render;
            case IRONPAGES_STATUS_FONT:
                return Status::font;
            case IRONPAGES_STATUS_IO:
                return Status::io;
            case IRONPAGES_STATUS_SECURITY:
                return Status::security;
            case IRONPAGES_STATUS_INTERNAL:
                return Status::internal;
            default:
                return Status::unknown;
        }
    }

    [[nodiscard]] static std::string message(const IronpagesError* error,
                                             IronpagesStatus status) {
        if (error == nullptr) {
            return "Ironpages failed with native status " +
                   std::to_string(status) + ".";
        }
        const auto size = ironpages_error_message_len(error);
        const auto* data = ironpages_error_message_data(error);
        if (data == nullptr || size == 0) {
            return "Ironpages returned an empty native diagnostic.";
        }
        return {reinterpret_cast<const char*>(data), size};
    }
};

[[nodiscard]] inline IronpagesBytes text(std::string_view value) noexcept {
    return {reinterpret_cast<const std::uint8_t*>(value.data()), value.size()};
}

}
