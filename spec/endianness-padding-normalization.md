AHD-1024 Endianness and Padding Normalization

Version: 1.0-pre
Status: Decision Document
Date: 2026-04-18

1. Problem Statement

The current implementation contains an implicit tension:

- Lane packing (64-bit words): Little-endian
- Bit ordering within bytes: MSB-first in prior wording
- Padding bit appending: ambiguous under mixed conventions

This creates ambiguity for independent implementers. A clean cryptographic specification must define exactly one canonical mapping from input bytes to state bits.

2. Decision: Consistent Little-Endian Worldview

Chosen convention: Little-endian throughout.

2.1 Rationale

- Implementation efficiency: matches native x86/ARM lane handling
- Rotation semantics: natural under LE lane interpretation
- FARD compatibility: single deterministic convention
- Padding simplicity: canonical byte-level form

2.2 What This Means

- Within a byte, bit 0 is the least significant bit
- A 64-bit lane is formed from 8 consecutive bytes in little-endian order
- Padding is expressed canonically as full bytes: 0x01 || 0x00* || 0x80

3. Canonical Mapping Specification

3.1 Input Bytes to Rate Blocks

- Rate = 1024 bits = 128 bytes
- Input bytes are processed in order
- The first 128 bytes form the first rate block

3.2 Rate Block to Lanes

Each 64-bit lane is decoded as:

Lane = bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | ... | (bytes[7] << 56)

3.3 Bit Indexing Within a Lane

- Bit 0 is the least significant bit
- Bit 63 is the most significant bit
- ROTL(L, k) moves bit i to (i + k) mod 64

3.4 Padding Specification (Canonical)

Let:

B = M || D

where M is the message and D is the domain byte.

Padding is:

1. Append one byte 0x01
2. Append zero bytes 0x00 until len(B) ≡ 127 (mod 128)
3. Append one byte 0x80

Equivalently, padded input is always:

M || D || 0x01 || 0x00* || 0x80

with final total length a multiple of 128 bytes.

Special cases:

- If len(M || D) % 128 == 127, append only 0x80 after 0x01’s placement logic is satisfied by the current block structure.
- If len(M || D) % 128 == 0, produce a full additional padding block.

4. Worked Example: Empty Message Hash

Input: M = ""
Domain: 0x01

First block bytes:

01 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 80

Lane 0 = 0x0000000000000101
Lane 1 = 0x0000000000000000

5. Worked Example: "abc"

Input: M = 61 62 63
Domain: 0x01

First bytes:

61 62 63 01 01 00 ... 00 80

6. Absorb Order Specification

Rate lanes occupy:

(0,0), (1,0), (2,0), (3,0), (4,0),
(0,1), (1,1), (2,1), (3,1), (4,1),
(0,2), (1,2), (2,2), (3,2), (4,2),
(0,3)

For each block:

1. Parse into 16 LE lanes
2. XOR into rate lanes in the listed order
3. Apply 24-round permutation

7. Squeeze Order Specification

For 256-bit digest output:

- Extract lanes (0,0), (1,0), (2,0), (3,0)
- Emit each as 8 bytes in little-endian order
- Concatenate

For XOF:

- Continue in rate-lane order
- Re-permute after exhausting rate lanes

8. Post-Normalization Boundary Vectors

HASH(zero126) = 370eece8418bab3710ce866b88a632c27537b80466c321e3f78faf43c55f3389
XOF64(zero126) = 8883158ab1e2d6b7075d81182d382ac40fabafa5d6bdffa1ec1201070654c6707ba06cfbc6637ca7574d41733eecd653298802826d7a97e3cceb4f8fc62e9866

9. Versioning

This document establishes the canonical v1.0 bit/byte convention.

Any future change to these conventions is a major-version change.
