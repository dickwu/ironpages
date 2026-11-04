#include "ironpages.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    static const char source[] = "<h1>Installed C package</h1>";
    const IronpagesBytes html = {
        .data = (const uint8_t *)source,
        .len = sizeof(source) - 1,
    };
    IronpagesBuffer *pdf = NULL;
    IronpagesError *error = NULL;

    if (ironpages_html_to_pdf(html, &pdf, &error) != IRONPAGES_STATUS_OK) {
        return EXIT_FAILURE;
    }
    if (ironpages_buffer_len(pdf) < 4 ||
        memcmp(ironpages_buffer_data(pdf), "%PDF", 4) != 0) {
        return EXIT_FAILURE;
    }
    if (ironpages_buffer_free(&pdf) != IRONPAGES_STATUS_OK) {
        return EXIT_FAILURE;
    }
    return EXIT_SUCCESS;
}
