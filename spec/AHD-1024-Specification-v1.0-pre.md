# AHD-1024 Specification

**Version:** v1.0-pre (Post-Normalization, Pre-Freeze)  
**Status:** Draft -- not yet frozen

Sections 1-10 are **normative**. Appendices A-B are informative.  
Examples are illustrative only unless explicitly identified as official test vectors.

---

## 1  Scope

This specification defines two cryptographic functions:

- **AHD-1024-256**: a hash function that accepts any finite byte string and produces a 32-byte digest.
- **AHD-1024-XOF**: an extendable output function that accepts any finite byte string and an output length L >= 0, and produces exactly L bytes.

Both functions are built on a 1600-bit sponge construction over a 24-round permutation with rate 1024 bits and capacity 576 bits.

The input domain is any byte string of length 0 or greater. Bit-string inputs of non-octet length are not defined by this specification.

---

## 2  Notation and Conventions

### 2.1  Glossary

| Term | Definition |
|------|------------|
| Sponge construction | A mode of operation for a fixed-length permutation that supports arbitrary-length input and output. Consists of an absorb phase and a squeeze phase separated by the permutation. |
| Permutation | A bijective function on a fixed-size state. AHD-1024 uses a 1600-bit permutation applied once per input block and once per output block beyond the first. |
| Rate | The portion of the state XORed with input (absorb) or extracted as output (squeeze). AHD-1024 rate = 1024 bits = 128 bytes. |
| Capacity | The portion of the state not directly exposed to input or output. AHD-1024 capacity = 576 bits. Determines generic security bounds. |
| Lane | A 64-bit word within the 5x5 state array. There are 25 lanes total. |
| Absorption | The process of XORing padded input blocks into the rate portion of the state, each followed by a permutation call. |
| Squeezing | The process of extracting output bytes from the rate portion of the state, applying the permutation between extractions when more output is needed. |
| Domain separation | A mechanism to ensure that two different modes (e.g. hash and XOF) applied to the same input produce independent outputs. Achieved here via a suffix byte appended before padding. |
| Padding | Bytes appended to the message to make its length a multiple of the rate. Ensures unambiguous parsing of any input. |
| Little-endian | Byte ordering where the least-significant byte appears first. All lane serialisation in this specification is little-endian. |
| ROL(v, n) | Left rotation of a 64-bit value v by n bit positions. Equivalent to (v << n) | (v >> (64 - n)), with n reduced mod 64. |
| Hash mode | AHD-1024-256: fixed 32-byte output, single squeeze without additional permutation calls. |
| XOF mode | AHD-1024-XOF: variable-length output, squeeze continues across permutation calls until L bytes are produced. |

### 2.2  Symbolic Notation

| Symbol     | Meaning |
|------------|---------|
| byte       | An octet: an integer in [0, 255] |
| M          | A message: a finite sequence of bytes, length |M| >= 0 |
| A[x][y]    | Lane at column x, row y; x,y in {0,1,2,3,4} |
| A[x][y][z] | Bit z of lane A[x][y]; z in {0,...,63} |
| ^          | Bitwise XOR |
| &          | Bitwise AND |
| ~          | Bitwise NOT |
| ROL(v,n)   | Left rotation of 64-bit value v by n positions; n reduced mod 64 |
| 0x...      | Hexadecimal literal |
| LE64(v)    | 64-bit value v serialised as 8 bytes, little-endian |

### 2.3  Bit and Byte Numbering

Within a 64-bit lane, bit 0 is the least-significant bit. Within a byte, bit 0 is the least-significant bit. Byte index 0 is the first byte of a sequence.

All multi-byte integers are little-endian unless otherwise stated.

---

## 3  Parameters

| Parameter                   | Value |
|-----------------------------|-------|
| State width                 | 1600 bits |
| Lane count                  | 25 (5 x 5 array) |
| Lane width                  | 64 bits |
| Rate (r)                    | 1024 bits = 128 bytes |
| Capacity (c)                | 576 bits = 72 bytes |
| Rounds per permutation call | 24 |
| Hash digest length          | 256 bits = 32 bytes |
| XOF output length           | L bytes, L >= 0 (caller-specified) |

---

## 4  State Representation

### 4.1  Logical State

The permutation state is a 5x5 array of 64-bit lanes A[x][y], with x (column) and y (row) each ranging over {0,1,2,3,4}.

### 4.2  Lane Index Mapping

Lane index i is computed from coordinates (x, y) as:

```
i = x + 5*y
```

Lane 0 is A[0][0]; lane 24 is A[4][4]. The 25 lanes are ordered by increasing index (i = 0 to 24).

The rate occupies lanes 0 to 15 (16 lanes x 8 bytes = 128 bytes).
The capacity occupies lanes 16 to 24.
The rate lanes in index order are:

| Lane i | A[x][y]  | Lane i | A[x][y]  |
|--------|----------|--------|----------|
|  0     | A[0][0]  |  8     | A[3][1]  |
|  1     | A[1][0]  |  9     | A[4][1]  |
|  2     | A[2][0]  | 10     | A[0][2]  |
|  3     | A[3][0]  | 11     | A[1][2]  |
|  4     | A[4][0]  | 12     | A[2][2]  |
|  5     | A[0][1]  | 13     | A[3][2]  |
|  6     | A[1][1]  | 14     | A[4][2]  |
|  7     | A[2][1]  | 15     | A[0][3]  |

### 4.3  Byte-to-State Mapping (Absorb)

Given a 128-byte rate block B, the first 128 bytes of the state are updated by XOR:

```
For i = 0 to 127:  state_byte(i)  ^=  B[i]
```

where state_byte(i) denotes byte (i mod 8) of lane floor(i / 8),
with each lane serialised as a little-endian 64-bit integer.

Equivalently: input byte B[j] is XORed into byte (j mod 8) of lane (j / 8),
where byte 0 of a lane is its least-significant byte.

Bytes beyond index 127 (capacity lanes 16-24) are never modified by absorption.

**Worked example:** Given B[0]=0x01, B[1]=0x02, ..., B[7]=0x08,
B[8]=0x09, ..., B[15]=0x10, all other bytes zero:

- B[0..7] map to lane 0 = A[0][0]:
  A[0][0] ^= 0x0807060504030201  (B[0] is the least-significant byte)
- B[8..15] map to lane 1 = A[1][0]:
  A[1][0] ^= 0x100f0e0d0c0b0a09

### 4.4  State-to-Byte Mapping (Squeeze)

To extract output bytes, serialise lanes in index order (i = 0, 1, 2, ...),
each as a little-endian 64-bit integer, and concatenate:

```
output_bytes = LE64(lane_0) || LE64(lane_1) || ... || LE64(lane_{n-1})
```

where lane_i corresponds to A[x][y] with x = i mod 5, y = floor(i / 5).

For hash mode (32 bytes), extract lanes 0-3:

```
LE64(A[0][0]) || LE64(A[1][0]) || LE64(A[2][0]) || LE64(A[3][0])
```

For XOF mode, continue extracting lanes in index order across permutation
calls until L bytes have been produced.

### 4.5  Initial State

The state is initialised to all-zero bits before the first absorption.

---

## 5  Domain Separation and Padding

### 5.1  Domain Suffix Bytes

| Mode         | Domain suffix byte (D) |
|--------------|------------------------|
| AHD-1024-256 | 0x01                   |
| AHD-1024-XOF | 0x01                   |

> **NOTE** Both modes currently use the same suffix byte. Future mode additions must use distinct values.

### 5.2  Padding Algorithm

Let r_bytes = 128. Given message M of m bytes, construct padded message P as follows:

```
1. Append byte D (the domain suffix for the selected mode).
2. Append the minimum number k of 0x00 bytes such that
   (m + 1 + k) mod r_bytes == r_bytes - 1.
3. Append byte 0x80.
```

The length of P is always a positive multiple of r_bytes.
The final byte of P is always 0x80.
The minimum padded length is r_bytes = 128 bytes (when m = 0).

### 5.3  Edge Cases

The following cases must be handled correctly:

| m mod r_bytes | k (zero bytes inserted) | Padded length |
|---------------|------------------------|---------------|
| 0             | 126                    | m + 128       |
| 126           | 0                      | m + 2         |
| 127           | 127                    | m + 129       |

> **NOTE** When m mod r_bytes = 126, the domain byte and 0x80 fill the
> last two positions exactly and k = 0. When m mod r_bytes = 127, only
> one byte remains in the block for the domain byte, so an entire extra
> block of padding is required.

### 5.4  Padding Examples

- m = 0 (empty):   P = [0x01, 0x00 x 126, 0x80], length 128
- m = 1:           P = M || [0x01, 0x00 x 125, 0x80], length 128
- m = 126:         P = M || [0x01, 0x80], length 128  (k = 0)
- m = 127:         P = M || [0x01, 0x00 x 126, 0x80], length 256
- m = 128:         P = M || [0x01, 0x00 x 126, 0x80], length 256
- m = 254:         P = M || [0x01, 0x80], length 256  (k = 0)
- m = 255:         P = M || [0x01, 0x00 x 126, 0x80], length 384

### 5.5  Unambiguity

The padding rule is prefix-free within each mode: no two distinct messages
produce the same padded block sequence. The domain suffix byte ensures that
hash mode and XOF mode inputs are distinct even for identical messages,
provided their domain bytes differ in future versions.

---

## 6  Absorb and Squeeze Procedures

### 6.1  Absorb

```
Input: message M of m bytes.

1. Construct P = pad(M)  (Section 5).
2. Split P into blocks B_0, B_1, ..., B_{n-1}, each of 128 bytes.
3. For i = 0 to n-1:
       XOR B_i into the first 128 state bytes  (Section 4.3).
       Apply the permutation  (Section 7).
```

### 6.2  Squeeze -- Hash Mode

```
After absorb, extract 32 bytes from the state (Section 4.4).
Return those 32 bytes as the digest. No further permutation calls are made.
```

### 6.3  Squeeze -- XOF Mode

```
Input: requested output length L >= 0 bytes.

1. Set output O = empty.
2. While |O| < L:
       Extract min(128, L - |O|) bytes from the state (Section 4.4).
       Append those bytes to O.
       If |O| < L: apply the permutation.
3. Return O.
```

> **NOTE** For L = 0, the procedure returns an empty byte string without applying the permutation.

---

## 7  Permutation

### 7.1  Round Function

The permutation applies 24 rounds. Each round applies five steps in order:

```
For ir = 0 to 23:
    A <- Theta(A)
    A <- Rho(A)
    A <- Pi(A)
    A <- ChiStar(A)
    A <- Iota(A, ir)
```

In all steps, outputs are computed from the input state to that step. No step reads partially updated values from the same step. Implementations must use a temporary copy or equivalent simultaneous-assignment strategy.

### 7.2  Theta

```
For x = 0 to 4:
    C[x] = A[x][0] ^ A[x][1] ^ A[x][2] ^ A[x][3] ^ A[x][4]

For x = 0 to 4:
    D[x] = C[(x+4) mod 5]
          ^ ROL(C[(x+1) mod 5], 1)
          ^ ROL(C[(x+1) mod 5], 8)
          ^ ROL(C[(x+1) mod 5], 57)

For x = 0 to 4, y = 0 to 4:
    A'[x][y] = A[x][y] ^ D[x]
```

### 7.3  Rho

Rho applies a fixed left rotation to each lane. Offsets are reduced mod 64. ROT[0][0] = 0 (A[0][0] is not rotated).

```
For x = 0 to 4, y = 0 to 4:
    A'[x][y] = ROL(A[x][y], ROT[x][y])
```

ROT table (row = y, column = x):

| y \ x |  0 |  1 |  2 |  3 |  4 |
|--------|----|----|----|----|-----|
|   0    |  0 | 36 |  3 | 41 | 18 |
|   1    |  1 | 44 | 10 | 45 |  2 |
|   2    | 62 |  6 | 43 | 15 | 61 |
|   3    | 28 | 55 | 25 | 21 | 56 |
|   4    | 27 | 20 | 39 |  8 | 14 |

### 7.4  Pi

```
For x = 0 to 4, y = 0 to 4:
    A'[x][y] = A[(x + 3*y) mod 5][x]
```

### 7.5  ChiStar

All right-hand-side values are taken from the input state to this step. Implementations MUST use a temporary row buffer -- A'[x][y] must not be written back before all five t-values for the row have been read.

```
For y = 0 to 4:
    For x = 0 to 4:
        t0 = A[x][y]
        t1 = A[(x+1) mod 5][y]
        t2 = A[(x+2) mod 5][y]
        t3 = A[(x+3) mod 5][y]
        t4 = A[(x+4) mod 5][y]
        A'[x][y] = t0 ^ (~t1 & t2) ^ (t1 & ~t3) ^ (~t2 & t4)
```

### 7.6  Iota

Iota XORs three round constants into specific lanes. All other lanes are unchanged.

```
A'[0][0] = A[0][0] ^ K0[ir]
A'[1][2] = A[1][2] ^ K1[ir]
A'[4][4] = A[4][4] ^ K2[ir]
```

K0, K1, K2 are defined in Section 8.

---

## 8  Round Constants

### 8.1  Normative Table

The following 72 values are normative. In any conflict between this table and the derivation procedure (Section 8.2), **this table governs**.

Constants are 64-bit values in hexadecimal, little-endian when serialised.

**K0 -- XORed into A[0][0]:**

| ir | K0[ir] |
|-------|----------------------|
|  0    | `0x1574243b711d5566` |
|  1    | `0x5295435425623498` |
|  2    | `0x981e63bd9227548f` |
|  3    | `0x85245ef134151de8` |
|  4    | `0x570b60f8a6c20187` |
|  5    | `0x825b702513673462` |
|  6    | `0x57394128a712edd5` |
|  7    | `0x88c6d2b560bbf31f` |
|  8    | `0xa7a8bf248bca25cb` |
|  9    | `0xeb180c1e04b2172d` |
| 10    | `0x4ef4c6faea19a9a7` |
| 11    | `0xf1b5dd76682c4d0e` |
| 12    | `0x3b6b72e42bc33ed2` |
| 13    | `0x89ed2667df0e851b` |
| 14    | `0x71f5a8ec1fe2e024` |
| 15    | `0x93415e15b9efef53` |
| 16    | `0x1f9b3eeb85f1f474` |
| 17    | `0xb04ae9a46d3a6472` |
| 18    | `0xd1192de0a0206232` |
| 19    | `0xbb78ca488168cda2` |
| 20    | `0xc680a672283ce9a7` |
| 21    | `0x017e3fde3366f029` |
| 22    | `0xde256a469bd72163` |
| 23    | `0x273399a3c1cffdc3` |

**K1 -- XORed into A[1][2]:**

| ir | K1[ir] |
|-------|----------------------|
|  0    | `0x8dfdeff5e7c5a8b9` |
|  1    | `0x388c559b10ae483d` |
|  2    | `0xe883fac12db91af0` |
|  3    | `0x8e7ef9a5616b0ac6` |
|  4    | `0x2dfff20e341431a3` |
|  5    | `0xeafc9706b51ef41a` |
|  6    | `0x68bb7e9df2ee8b1e` |
|  7    | `0x4b10554a0372f85b` |
|  8    | `0xd327ee76693b2528` |
|  9    | `0xe00339e4ad409702` |
| 10    | `0x217665d3dd485b86` |
| 11    | `0xbae752b02854e952` |
| 12    | `0xbf1661eee2376cf5` |
| 13    | `0x262f4015d7f3806f` |
| 14    | `0x826bde510a3f4b3f` |
| 15    | `0xf000fb5050b7061f` |
| 16    | `0x72b2c51a3a095e01` |
| 17    | `0x7086248d2add1c7d` |
| 18    | `0x696903648c61f519` |
| 19    | `0x0f0a90965c95658f` |
| 20    | `0xa84015c021a9556f` |
| 21    | `0x71b85d2e447dfd37` |
| 22    | `0x513b79b5a657f550` |
| 23    | `0x543757c1718c7f4b` |

**K2 -- XORed into A[4][4]:**

| ir | K2[ir] |
|-------|----------------------|
|  0    | `0xcf4c8a555ab20a8c` |
|  1    | `0x0074aa1d4fbfdc52` |
|  2    | `0x2c14aaaa44941c49` |
|  3    | `0x281f0a9f3ddf5f77` |
|  4    | `0x05de9e119ccb9d41` |
|  5    | `0xd10891162cdea6dc` |
|  6    | `0x4065787be1d7f474` |
|  7    | `0x6d33c6d5fd71489e` |
|  8    | `0x06464be296e21f7a` |
|  9    | `0x28ed230c533fd4b1` |
| 10    | `0x499f4879e167454c` |
| 11    | `0x0ab60623de9f9f5a` |
| 12    | `0x1d89d5f9797558e9` |
| 13    | `0x0f28c9ccd69f91bd` |
| 14    | `0x1e4d3614370c5648` |
| 15    | `0x48fccad651d25277` |
| 16    | `0x520028a6e780206f` |
| 17    | `0x5ec839a25a743f5c` |
| 18    | `0x25afc0eefb608b23` |
| 19    | `0x3e067685a27bebad` |
| 20    | `0xc435b273ba5fd45e` |
| 21    | `0x15482bfb2b577f4a` |
| 22    | `0x1b6dbaf7b57710c8` |
| 23    | `0xed56c469d594e2b3` |

### 8.2  Derivation Procedure (Informative)

Constants are derived deterministically from SHAKE256. This derivation is informative only; the normative values are the frozen table in Section 8.1.

```
seed     = UTF-8 encoding of "AHA-D-256-ROUND-CONSTANTS-v0.1"
material = SHAKE256(seed, 576 bytes)
Parse material as 72 little-endian 64-bit values: v[0], v[1], ..., v[71]
K0[ir]  = v[ir]       for ir = 0 to 23
K1[ir]  = v[24 + ir]  for ir = 0 to 23
K2[ir]  = v[48 + ir]  for ir = 0 to 23
```

---

## 9  Algorithm Identifiers and Mode Definitions

### 9.1  AHD-1024-256 (Hash Mode)

```
Identifier: AHD-1024-256
Input:      byte string M, |M| >= 0
Output:     32 bytes

1. Initialise state to all zeros.
2. Absorb M with domain suffix 0x01  (Section 6.1).
3. Squeeze 32 bytes  (Section 6.2).
4. Return the 32 bytes.
```

### 9.2  AHD-1024-XOF (Extendable Output Mode)

```
Identifier: AHD-1024-XOF
Input:      byte string M, |M| >= 0; output length L >= 0
Output:     L bytes

1. Initialise state to all zeros.
2. Absorb M with domain suffix 0x01  (Section 6.1).
3. Squeeze L bytes  (Section 6.3).
4. Return the L bytes.
```

> **NOTE** L = 0 is valid and returns an empty byte string.

---

## 10  Conformance

### 10.1  Requirements

A conforming implementation of AHD-1024-256 or AHD-1024-XOF:

- MUST produce identical outputs for all valid inputs as defined by this specification.
- MUST use the exact padding rule defined in Section 5.
- MUST use the exact state mapping defined in Section 4.
- MUST use the exact round constants defined in Section 8.1.
- MUST use the exact round function defined in Section 7.
- MUST pass all official test vectors defined in Section 11.
- MUST NOT reject any valid input (a finite byte string of length >= 0).
- MUST support messages of at least 2^32 - 1 bytes. Implementations with lower practical limits must document those limits explicitly.

### 10.2  Permissions

A conforming implementation:

- MAY process input incrementally (streaming).
- MAY use SIMD, bitsliced, table-based, or hardware-accelerated implementations of any step, provided outputs are identical.
- MAY return output in hexadecimal or other encodings provided the raw bytes match this specification.
- MAY allocate state on the stack or heap.

---

## 11  Official Test Vectors

The following are normative known-answer vectors. All conforming implementations must produce identical outputs.

### 11.1  Hash Mode (AHD-1024-256)

| Input | Length (bytes) | Digest (hex, 32 bytes) |
|-------|---------------|------------------------|
| empty | 0 | `e8bf66fb70ec3787817c0cb717952140569a853f94dee36a21268632b9a59ed0` |
| "a" | 1 | `ef258013d45d8f04fc2d6364a54a48c008391c81811cb9ab9ca9a2be4df90bbe` |
| "abc" | 3 | `50f4f48736c87a32bb20c618fda7de0ec0260edd57f340e92d8daa45d54a4a1f` |
| a-z A-Z | 52 | `9145c3bcd241cc8347a8d55fc41990ded5b2d5e062cc510deb91f78903a35b09` |
| 0x00 x 1 | 1 | `3bbff54e02c3149faba2b629393b06ded81ef16b50282543d90ec3d702b7e86d` |
| 0x00 x 126 | 126 | `370eece8418bab3710ce866b88a632c27537b80466c321e3f78faf43c55f3389` |
| 0x00 x 127 | 127 | `d75422b1f7b15494f7428fbd4911c8178e363f82032258c7180c98ce6fb1ba41` |
| 0x00 x 128 | 128 | `22598b6298b7125bdacf7486508d3efc34e93334f93b889b736e2614cd3479fe` |
| 0x00 x 129 | 129 | `183a22b19dd510e33fbc53b2066e16da807d00f16b900a43c33a1186498b8312` |
| 0x00 x 254 | 254 | `79f17f07809441f608ef07d9e8eff530e4d4eca1a82a5af399703df42979e63a` |
| 0x00 x 255 | 255 | `bbf6687a0643a3adee2c9e97d1b34a51126ae1149bf19ced85753f8c563fdd97` |
| 0x00 x 256 | 256 | `3487f4f24dd518db4270092c8cc266dd31ffeef0850789bc65705f2f4a2b14b1` |
| 0xff x 128 | 128 | `68513f624ee201a93aa39d4aa9a8d4221f5ea2a68d7fd5a91e9bcf686099e2f7` |
| 0x00 x 383 | 383 | `ff460408f71fff09d6109aad30d833727113d2467e9f17c5d5124a19559e9d09` |
| 0x00 x 384 | 384 | `b12678ce34d7cb93532f1e14549c722bd9b2cc4b09798f3865a75b6a94e41ff4` |
| counting 0x00-0xff | 256 | `a40db6771aa9b1856911dc70055a54926975b1385c2af5c35e2dada38412c5d7` |

> **NOTE** Boundary lengths 126, 127 (last byte of block), 128 (full block),
> 129 (one byte into next block), 254, 255, 256, 383, 384 exercise all
> padding edge cases defined in Section 5.3.

### 11.2  XOF Mode (AHD-1024-XOF)

#### 11.2.1  Variable output lengths, empty input

| L (bytes) | Output (hex) |
|-----------|-------------|
| 0 | (empty) |
| 1 | `01` |
| 32 | `01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f084` |
| 64 | `01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76` |
| 128 | `01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76f058ba42a17772ad14784c2e081fdf59d9f45bc21baa14e039b4a917f3e5b563df0b72611bb2b6065a65bf3422a69f095b14edfc1cb858bc758e77bd7201631d` |
| 129 | `01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76f058ba42a17772ad14784c2e081fdf59d9f45bc21baa14e039b4a917f3e5b563df0b72611bb2b6065a65bf3422a69f095b14edfc1cb858bc758e77bd7201631d39` |

#### 11.2.2  Variable output lengths, input "abc"

| L (bytes) | Output (hex) |
|-----------|-------------|
| 0 | (empty) |
| 1 | `87` |
| 32 | `87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891` |
| 64 | `87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f5410` |
| 128 | `87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f54107a81062f4cccf78d913e8c39e65d3cd9b67dbfa0c07e0c699f5caf5cc5f12e65a91f19fb7cd501b8832eb969c83d4c50a9a8e47747239217e4e49e631c04eade` |
| 129 | `87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f54107a81062f4cccf78d913e8c39e65d3cd9b67dbfa0c07e0c699f5caf5cc5f12e65a91f19fb7cd501b8832eb969c83d4c50a9a8e47747239217e4e49e631c04eadee1` |

> **NOTE** L=0 returns empty. L=129 exercises the squeeze across a rate boundary
> (128-byte rate requires a second permutation call for the 129th byte).
> Longer outputs are prefixes of shorter ones -- XOF output is a stream.

---

## Appendix A  Security Claims (Informative)

### A.1  Construction-Level Claims

AHD-1024 uses a sponge construction. Under the ideal-permutation assumption and standard sponge analysis, generic attacks are bounded by the capacity. With c = 576 bits, the generic collision bound is approximately 2^288 and the generic preimage bound is approximately 2^256 for the 256-bit hash output.

The construction does not exhibit Merkle-Damgard length extension.

### A.2  Empirical Evidence

The following results support confidence in the permutation. They do not constitute proofs of security.

- **Differential screens:** no low-weight trail survives beyond round 2 in any tested probe (200,000 pairs, multiple seeds).
- **Avalanche:** rounds 3-24 exhibit mean flip probability ~0.5000, max deviation < 0.004 at 320,000 samples.
- **ANF degree:** reaches maximum (16/16) by round 4 in the 4-variable exact subspace (lane_width=4, tracked_outputs=1).
- **Rotation symmetry:** zero nontrivial survivals across 200,000 samples.
- **Fixed points and short cycles:** none found across 200,000 samples through 6 rounds.
- **Beam search (2- and 3-bit seeds):** best round-3 trail weight ~738-747 bits.

These results support confidence in the permutation design. They do not prove collision or preimage resistance of the hash construction.

---

## Appendix B  Version and Stability (Informative)

**Current version:** v1.0-pre (post-normalization, pre-freeze)

Any change to padding, endianness, constants, or round structure requires a new major version. The identifiers `AHD-1024-256` and `AHD-1024-XOF` refer exclusively to the parameters and procedures defined in this document at the version stated above.

Three independent implementations (Rust, Python, C) produce bit-identical outputs for all test vectors in Section 11. The Rust implementation is the primary reference.

This document is the sole normative reference. The specification remains valid and sufficient for independent implementation even if the repository is unavailable.

---

*End of AHD-1024 Specification v1.0-pre*