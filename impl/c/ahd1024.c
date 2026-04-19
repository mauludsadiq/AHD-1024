#include "ahd1024.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define AHD1024_MASK64 UINT64_C(0xFFFFFFFFFFFFFFFF)

static const uint8_t AHD1024_SEED[] = "AHA-D-256-ROUND-CONSTANTS-v0.1";

static const uint32_t AHD1024_ROT[5][5] = {
    {0, 7, 19, 41, 53},
    {11, 29, 43, 3, 31},
    {37, 59, 5, 17, 47},
    {23, 13, 61, 27, 9},
    {45, 21, 39, 49, 55},
};

typedef uint64_t ahd1024_state_t[5][5];

static uint64_t ahd1024_rotl64(uint64_t x, uint32_t n) {
    n &= 63u;
    return (x << n) | (x >> (64u - n));
}

static uint64_t ahd1024_load64_le(const uint8_t *p) {
    return ((uint64_t)p[0]) |
           ((uint64_t)p[1] << 8) |
           ((uint64_t)p[2] << 16) |
           ((uint64_t)p[3] << 24) |
           ((uint64_t)p[4] << 32) |
           ((uint64_t)p[5] << 40) |
           ((uint64_t)p[6] << 48) |
           ((uint64_t)p[7] << 56);
}

static void ahd1024_store64_le(uint8_t *out, uint64_t x) {
    out[0] = (uint8_t)(x);
    out[1] = (uint8_t)(x >> 8);
    out[2] = (uint8_t)(x >> 16);
    out[3] = (uint8_t)(x >> 24);
    out[4] = (uint8_t)(x >> 32);
    out[5] = (uint8_t)(x >> 40);
    out[6] = (uint8_t)(x >> 48);
    out[7] = (uint8_t)(x >> 56);
}

static void ahd1024_blank_state(ahd1024_state_t s) {
    memset(s, 0, sizeof(ahd1024_state_t));
}

/* Minimal SHAKE256 implementation path via Python-generated constants is deferred.
   For Phase 2 parity, embed the currently frozen deterministic constants derived
   from the canonical seed. */

static const uint64_t AHD1024_K0[AHD1024_ROUNDS] = {
    UINT64_C(0x1574243b711d5566),
    UINT64_C(0x85245ef134151de8),
    UINT64_C(0x57394128a712edd5),
    UINT64_C(0xeb180c1e04b2172d),
    UINT64_C(0x3b6b72e42bc33ed2),
    UINT64_C(0x93415e15b9efef53),
    UINT64_C(0xd1192de0a0206232),
    UINT64_C(0x017e3fde3366f029),
    UINT64_C(0x8dfdeff5e7c5a8b9),
    UINT64_C(0x8e7ef9a5616b0ac6),
    UINT64_C(0x68bb7e9df2ee8b1e),
    UINT64_C(0xe00339e4ad409702),
    UINT64_C(0xbf1661eee2376cf5),
    UINT64_C(0xf000fb5050b7061f),
    UINT64_C(0x696903648c61f519),
    UINT64_C(0x71b85d2e447dfd37),
    UINT64_C(0xcf4c8a555ab20a8c),
    UINT64_C(0x281f0a9f3ddf5f77),
    UINT64_C(0x4065787be1d7f474),
    UINT64_C(0x28ed230c533fd4b1),
    UINT64_C(0x1d89d5f9797558e9),
    UINT64_C(0x48fccad651d25277),
    UINT64_C(0x25afc0eefb608b23),
    UINT64_C(0x15482bfb2b577f4a)
};

static const uint64_t AHD1024_K1[AHD1024_ROUNDS] = {
    UINT64_C(0x5295435425623498),
    UINT64_C(0x570b60f8a6c20187),
    UINT64_C(0x88c6d2b560bbf31f),
    UINT64_C(0x4ef4c6faea19a9a7),
    UINT64_C(0x89ed2667df0e851b),
    UINT64_C(0x1f9b3eeb85f1f474),
    UINT64_C(0xbb78ca488168cda2),
    UINT64_C(0xde256a469bd72163),
    UINT64_C(0x388c559b10ae483d),
    UINT64_C(0x2dfff20e341431a3),
    UINT64_C(0x4b10554a0372f85b),
    UINT64_C(0x217665d3dd485b86),
    UINT64_C(0x262f4015d7f3806f),
    UINT64_C(0x72b2c51a3a095e01),
    UINT64_C(0x0f0a90965c95658f),
    UINT64_C(0x513b79b5a657f550),
    UINT64_C(0x0074aa1d4fbfdc52),
    UINT64_C(0x05de9e119ccb9d41),
    UINT64_C(0x6d33c6d5fd71489e),
    UINT64_C(0x499f4879e167454c),
    UINT64_C(0x0f28c9ccd69f91bd),
    UINT64_C(0x520028a6e780206f),
    UINT64_C(0x3e067685a27bebad),
    UINT64_C(0x1b6dbaf7b57710c8)
};

static const uint64_t AHD1024_K2[AHD1024_ROUNDS] = {
    UINT64_C(0x981e63bd9227548f),
    UINT64_C(0x825b702513673462),
    UINT64_C(0xa7a8bf248bca25cb),
    UINT64_C(0xf1b5dd76682c4d0e),
    UINT64_C(0x71f5a8ec1fe2e024),
    UINT64_C(0xb04ae9a46d3a6472),
    UINT64_C(0xc680a672283ce9a7),
    UINT64_C(0x273399a3c1cffdc3),
    UINT64_C(0xe883fac12db91af0),
    UINT64_C(0xeafc9706b51ef41a),
    UINT64_C(0xd327ee76693b2528),
    UINT64_C(0xbae752b02854e952),
    UINT64_C(0x826bde510a3f4b3f),
    UINT64_C(0x7086248d2add1c7d),
    UINT64_C(0xa84015c021a9556f),
    UINT64_C(0x543757c1718c7f4b),
    UINT64_C(0x2c14aaaa44941c49),
    UINT64_C(0xd10891162cdea6dc),
    UINT64_C(0x06464be296e21f7a),
    UINT64_C(0x0ab60623de9f9f5a),
    UINT64_C(0x1e4d3614370c5648),
    UINT64_C(0x5ec839a25a743f5c),
    UINT64_C(0xc435b273ba5fd45e),
    UINT64_C(0xed56c469d594e2b3)
};

void ahd1024_derive_constants(ahd1024_constants_t *out) {
    memcpy(out->k0, AHD1024_K0, sizeof(AHD1024_K0));
    memcpy(out->k1, AHD1024_K1, sizeof(AHD1024_K1));
    memcpy(out->k2, AHD1024_K2, sizeof(AHD1024_K2));
}

static void ahd1024_theta(const ahd1024_state_t s, ahd1024_state_t out) {
    uint64_t c[5];
    uint64_t d[5];
    size_t x, y;

    for (x = 0; x < 5; ++x) {
        c[x] = s[x][0] ^ s[x][1] ^ s[x][2] ^ s[x][3] ^ s[x][4];
    }
    for (x = 0; x < 5; ++x) {
        d[x] = ahd1024_rotl64(c[(x + 4) % 5], 1) ^
               ahd1024_rotl64(c[(x + 1) % 5], 11) ^
               ahd1024_rotl64(c[(x + 2) % 5], 27);
    }
    for (x = 0; x < 5; ++x) {
        for (y = 0; y < 5; ++y) {
            out[x][y] = s[x][y] ^ d[x];
        }
    }
}

static void ahd1024_pi_stage(const ahd1024_state_t s, ahd1024_state_t out) {
    size_t x, y;
    for (x = 0; x < 5; ++x) {
        for (y = 0; y < 5; ++y) {
            out[x][y] = s[(2 * x + 3 * y) % 5][(x + 2 * y) % 5];
        }
    }
}

static void ahd1024_rho(const ahd1024_state_t s, ahd1024_state_t out) {
    size_t x, y;
    for (x = 0; x < 5; ++x) {
        for (y = 0; y < 5; ++y) {
            out[x][y] = ahd1024_rotl64(s[x][y], AHD1024_ROT[x][y]);
        }
    }
}

static void ahd1024_chi_star(const ahd1024_state_t s, ahd1024_state_t out) {
    size_t y, i;
    for (y = 0; y < 5; ++y) {
        uint64_t a[5] = {s[0][y], s[1][y], s[2][y], s[3][y], s[4][y]};
        for (i = 0; i < 5; ++i) {
            out[i][y] = (
                a[i] ^
                ((~a[(i + 1) % 5] & AHD1024_MASK64) & a[(i + 2) % 5]) ^
                (ahd1024_rotl64(a[(i + 3) % 5], 1) & ahd1024_rotl64(a[(i + 4) % 5], 3))
            ) & AHD1024_MASK64;
        }
    }
}

static void ahd1024_iota(const ahd1024_state_t s, size_t t, const ahd1024_constants_t *constants, ahd1024_state_t out) {
    memcpy(out, s, sizeof(ahd1024_state_t));
    out[0][0] ^= constants->k0[t];
    out[1][2] ^= constants->k1[t];
    out[4][4] ^= constants->k2[t];
}

static void ahd1024_permute(ahd1024_state_t s, size_t rounds, const ahd1024_constants_t *constants) {
    size_t t;
    ahd1024_state_t tmp;

    for (t = 0; t < rounds; ++t) {
        ahd1024_theta(s, tmp);
        memcpy(s, tmp, sizeof(ahd1024_state_t));
        ahd1024_pi_stage(s, tmp);
        memcpy(s, tmp, sizeof(ahd1024_state_t));
        ahd1024_rho(s, tmp);
        memcpy(s, tmp, sizeof(ahd1024_state_t));
        ahd1024_chi_star(s, tmp);
        memcpy(s, tmp, sizeof(ahd1024_state_t));
        ahd1024_iota(s, t, constants, tmp);
        memcpy(s, tmp, sizeof(ahd1024_state_t));
    }
}

static uint8_t *ahd1024_pad_v02(const uint8_t *message, size_t message_len, ahd1024_domain_t domain, size_t *padded_len_out) {
    size_t cap = message_len + 2 + AHD1024_RATE_BYTES;
    uint8_t *out = (uint8_t *)malloc(cap);
    size_t len = 0;

    if (out == NULL) {
        *padded_len_out = 0;
        return NULL;
    }

    if (message_len > 0) {
        memcpy(out, message, message_len);
        len += message_len;
    }

    out[len++] = (uint8_t)domain;
    out[len++] = 0x01;

    while ((len % AHD1024_RATE_BYTES) != (AHD1024_RATE_BYTES - 1)) {
        out[len++] = 0x00;
    }

    out[len++] = 0x80;
    *padded_len_out = len;
    return out;
}

static void ahd1024_absorb_blocks(
    const uint8_t *padded,
    size_t padded_len,
    size_t rounds,
    const ahd1024_constants_t *constants,
    ahd1024_state_t s
) {
    size_t block_start, i;
    ahd1024_blank_state(s);

    for (block_start = 0; block_start < padded_len; block_start += AHD1024_RATE_BYTES) {
        const uint8_t *block = padded + block_start;
        for (i = 0; i < 16; ++i) {
            uint64_t lane = ahd1024_load64_le(block + (8 * i));
            size_t x = i % 5;
            size_t y = i / 5;
            s[x][y] ^= lane;
        }
        ahd1024_permute(s, rounds, constants);
    }
}

static void ahd1024_squeeze_bytes(
    ahd1024_state_t s,
    size_t out_len,
    size_t rounds,
    const ahd1024_constants_t *constants,
    uint8_t *out
) {
    size_t written = 0;
    while (written < out_len) {
        size_t i;
        for (i = 0; i < 16 && written < out_len; ++i) {
            size_t x = i % 5;
            size_t y = i / 5;
            uint8_t lane_bytes[8];
            size_t take, j;

            ahd1024_store64_le(lane_bytes, s[x][y]);
            take = (out_len - written < 8) ? (out_len - written) : 8;
            for (j = 0; j < take; ++j) {
                out[written + j] = lane_bytes[j];
            }
            written += take;
        }
        if (written < out_len) {
            ahd1024_permute(s, rounds, constants);
        }
    }
}

void ahd1024_hash(
    const uint8_t *message,
    size_t message_len,
    ahd1024_domain_t domain,
    size_t out_len,
    uint8_t *out
) {
    ahd1024_constants_t constants;
    ahd1024_state_t s;
    uint8_t *padded;
    size_t padded_len = 0;

    ahd1024_derive_constants(&constants);
    padded = ahd1024_pad_v02(message, message_len, domain, &padded_len);
    if (padded == NULL) {
        return;
    }

    ahd1024_absorb_blocks(padded, padded_len, AHD1024_ROUNDS, &constants, s);
    ahd1024_squeeze_bytes(s, out_len, AHD1024_ROUNDS, &constants, out);

    free(padded);
}
