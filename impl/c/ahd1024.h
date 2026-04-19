#ifndef AHD1024_H
#define AHD1024_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define AHD1024_RATE_BITS 1024
#define AHD1024_RATE_BYTES (AHD1024_RATE_BITS / 8)
#define AHD1024_ROUNDS 24

typedef enum {
    AHD1024_DOMAIN_HASH = 0x01,
    AHD1024_DOMAIN_XOF = 0x02,
    AHD1024_DOMAIN_TREE_LEAF = 0x03,
    AHD1024_DOMAIN_TREE_PARENT = 0x04,
    AHD1024_DOMAIN_MAC_KEYED = 0x05,
    AHD1024_DOMAIN_TRANSCRIPT = 0x06,
    AHD1024_DOMAIN_ARTIFACT = 0x07,
    AHD1024_DOMAIN_ROUND_TRACE = 0x08
} ahd1024_domain_t;

typedef struct {
    uint64_t k0[AHD1024_ROUNDS];
    uint64_t k1[AHD1024_ROUNDS];
    uint64_t k2[AHD1024_ROUNDS];
} ahd1024_constants_t;

void ahd1024_derive_constants(ahd1024_constants_t *out);

void ahd1024_hash(
    const uint8_t *message,
    size_t message_len,
    ahd1024_domain_t domain,
    size_t out_len,
    uint8_t *out
);

#ifdef __cplusplus
}
#endif

#endif
