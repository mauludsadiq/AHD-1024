#include "ahd1024.h"

#include <stdio.h>
#include <string.h>

static void print_hex(const uint8_t *buf, size_t len) {
    size_t i;
    for (i = 0; i < len; ++i) {
        printf("%02x", buf[i]);
    }
    printf("\n");
}

int main(void) {
    uint8_t out[64];

    ahd1024_hash((const uint8_t *)"", 0, AHD1024_DOMAIN_HASH, 32, out);
    print_hex(out, 32);

    ahd1024_hash((const uint8_t *)"abc", 3, AHD1024_DOMAIN_HASH, 32, out);
    print_hex(out, 32);

    return 0;
}
