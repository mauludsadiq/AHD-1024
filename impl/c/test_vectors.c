#include "ahd1024.h"

#include <stdio.h>
#include <stdint.h>
#include <string.h>

static void hex_encode(const uint8_t *in, size_t len, char *out) {
    static const char *hex = "0123456789abcdef";
    size_t i;
    for (i = 0; i < len; ++i) {
        out[2 * i] = hex[(in[i] >> 4) & 0xF];
        out[2 * i + 1] = hex[in[i] & 0xF];
    }
    out[2 * len] = '\0';
}

static int check_case(const char *name, const uint8_t *msg, size_t msg_len,
                      const char *exp_hash, const char *exp_xof) {
    uint8_t out32[32];
    uint8_t out64[64];
    char got32[65];
    char got64[129];
    int ok = 1;

    ahd1024_hash(msg, msg_len, AHD1024_DOMAIN_HASH, 32, out32);
    ahd1024_hash(msg, msg_len, AHD1024_DOMAIN_XOF, 64, out64);

    hex_encode(out32, 32, got32);
    hex_encode(out64, 64, got64);

    if (strcmp(got32, exp_hash) != 0) {
        printf("HASH %s: FAIL\n  expected: %s\n  got     : %s\n", name, exp_hash, got32);
        ok = 0;
    } else {
        printf("HASH %s: OK\n", name);
    }

    if (strcmp(got64, exp_xof) != 0) {
        printf("XOF64 %s: FAIL\n  expected: %s\n  got     : %s\n", name, exp_xof, got64);
        ok = 0;
    } else {
        printf("XOF64 %s: OK\n", name);
    }

    return ok;
}

int main(void) {
    uint8_t zero126[126] = {0};
    uint8_t zero128[128] = {0};
    uint8_t ff128[128];
    memset(ff128, 0xff, sizeof(ff128));

    int ok = 1;

    ok &= check_case(
        "empty", (const uint8_t *)"", 0,
        "e8bf66fb70ec3787817c0cb717952140569a853f94dee36a21268632b9a59ed0",
        "01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76"
    );

    ok &= check_case(
        "a", (const uint8_t *)"a", 1,
        "ef258013d45d8f04fc2d6364a54a48c008391c81811cb9ab9ca9a2be4df90bbe",
        "aeda390a8c1cdde1466e780c8ff7d0fd83dcaa005c14b0793420f18a95041e1c6234b194c8b4125cfbfbead050ff4e56c3f0de5ceb8ae1c47cef4ffa8ec27077"
    );

    ok &= check_case(
        "abc", (const uint8_t *)"abc", 3,
        "50f4f48736c87a32bb20c618fda7de0ec0260edd57f340e92d8daa45d54a4a1f",
        "87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f5410"
    );

    ok &= check_case(
        "alphabet52",
        (const uint8_t *)"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ", 52,
        "9145c3bcd241cc8347a8d55fc41990ded5b2d5e062cc510deb91f78903a35b09",
        "762507d70670d740bbcd11ae38a652a3775a9f7f87215eeb54c95e11f9385b6c2278639f387afc1da8534c4da8d7e1b3be74021b2d07439eaf733d3e04567104"
    );

    ok &= check_case(
        "zero126", zero126, sizeof(zero126),
        "370eece8418bab3710ce866b88a632c27537b80466c321e3f78faf43c55f3389",
        "8883158ab1e2d6b7075d81182d382ac40fabafa5d6bdffa1ec1201070654c6707ba06cfbc6637ca7574d41733eecd653298802826d7a97e3cceb4f8fc62e9866"
    );

    ok &= check_case(
        "zero128", zero128, sizeof(zero128),
        "22598b6298b7125bdacf7486508d3efc34e93334f93b889b736e2614cd3479fe",
        "172de4305d9c7ff04042c4ea69fd3602588148f0b998f6de4b6769def6a8f41ff40709b812f143d688265d2ae4d66afa581c22d53e57e7211a9694064694ced1"
    );

    ok &= check_case(
        "ff128", ff128, sizeof(ff128),
        "68513f624ee201a93aa39d4aa9a8d4221f5ea2a68d7fd5a91e9bcf686099e2f7",
        "6b5506e49a8cdd0cf99ffd934718a6abae6d6bbbcf725f40a82b2112f5dd66479a854a231af3e65b4cf12c489bf0be8773ff45ceb40d5d2af2d66bde2913cc2e"
    );

    printf("%s\n", ok ? "ALL_OK" : "MISMATCH");
    return ok ? 0 : 1;
}
