#include "ironpages.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    static const char source[] = "<h1>Installed with Conan</h1>";
    const IronpagesBytes html = {(const uint8_t *)source, sizeof(source) - 1};
    IronpagesBuffer *pdf = NULL;
    IronpagesError *error = NULL;

    if (ironpages_html_to_pdf(html, &pdf, &error) != IRONPAGES_STATUS_OK) {
        ironpages_error_free(&error);
        return EXIT_FAILURE;
    }
    const int valid = ironpages_buffer_len(pdf) >= 4 &&
                      memcmp(ironpages_buffer_data(pdf), "%PDF", 4) == 0;
    ironpages_buffer_free(&pdf);
    return valid ? EXIT_SUCCESS : EXIT_FAILURE;
}
