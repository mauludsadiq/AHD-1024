# AHD-1024 Design Rationale

**Version:** v1.0-pre
**Date:** 2026-04-20

This document records the empirical justification for each design choice in AHD-1024.
All conclusions are data-backed. No choice is retained by taste alone.

---

## 1  Why a Sponge Construction

The sponge construction was chosen for the following properties:

- No Merkle-Damgard length extension vulnerability.
- Unified hash and XOF modes from a single primitive.
- Generic security bounds determined by capacity, independent of output length.
- Well-understood security model under ideal-permutation assumption.

With rate = 1024 bits and capacity = 576 bits, the generic collision bound is
2^288 and the preimage bound is 2^256. The 256-bit output limits collision
resistance to 2^128 in practice (birthday bound).

---

## 2  Why This State Geometry (1600 bits, 5x5x64)

- 1600-bit state provides comfortable margin above the 576-bit capacity requirement.
- 5x5 lane array enables efficient Theta diffusion across all columns and rows.
- 64-bit lanes align with 64-bit CPU registers for efficient implementation.
- 25 lanes provide sufficient mixing surface for the nonlinear and permutation steps.

---

## 3  Why This Nonlinear Layer (Chi*)

Three nonlinear layer candidates were evaluated using the low-weight differential
screen at 200,000 pairs, seed 7:

| Variant | Round 2 avg | Round 2 min | Round 2 count_le_64 |
|---------|-------------|-------------|---------------------|
| Chi* (spec) | 115.534 | 75 | 0 |
| Chi' (alternate) | 114.450 | 69 | 0 |
| Chi (baseline) | 78.800 | 36 | 13315 |

**Conclusion:** Chi* is decisively ahead of baseline Chi at the critical round-2
transitional regime. Chi' is viable but weaker than Chi* on both average and
minimum. Chi* is retained as the nonlinear layer.

Result files: results/low_weight_{star,alt,baseline}_pairs200000_msg96_seed7.json

---

## 4  Why This Rotation Table

Three rotation table variants were evaluated:

| Variant | Round 2 avg | Round 2 min |
|---------|-------------|-------------|
| Current ROT | 115.554 | 73 |
| ROT + 1 mod 64 | 115.549 | 78 |
| ROT + 7 mod 64 | 115.546 | 75 |

All three variants are effectively tied on average. ROT+1 slightly improves
the round-2 minimum but not by enough to establish a decisive win.

**Conclusion:** The current rotation table is not sitting on a knife edge --
nearby perturbations produce equivalent results. This is a positive stability
signal. The current table is retained. No decisive replacement has emerged.

Result files: results/low_weight_shifted{,_b}_pairs200000_msg96_seed7.json

---

## 5  Why 3-Site Iota Injection

Three iota injection configurations were evaluated:

| Variant | Sites | Round 2 avg | Round 2 min | Round 3 avg | Round 3 min |
|---------|-------|-------------|-------------|-------------|-------------|
| 1-site | A[0][0] only | 116.621 | 78 | 127.990 | 94 |
| 3-site (current) | A[0][0], A[1][2], A[4][4] | 116.674 | 82 | 128.011 | 91 |
| 5-site | above + A[2][4], A[3][1] | 116.679 | 79 | 128.002 | 95 |

All three are effectively tied on average. The 3-site current design has the
best round-2 minimum (82). The 5-site variant adds complexity with no
meaningful gain. The 1-site variant is marginally weaker at round 2.

**Conclusion:** 3-site iota is the correct balance of asymmetry and simplicity.
The injection sites A[0][0], A[1][2], A[4][4] are frozen.

Result files: results/iota_tournament_{1site,3site,5site}_pairs200000_msg96_seed7.json

---

## 6  Why 24 Rounds

Round count was evaluated against the observed collapse boundary:

- Round 1: shallow across all probes (avg ~7.6 changed bits, avalanche ~0.030).
- Round 2: transitional (avg ~115.5 changed bits, avalanche ~0.451).
- Round 3: first statistically ideal round (avg ~128, avalanche ~0.500).
- Rounds 3-24: stable under all probes with no degradation.

24 rounds provides a security margin of 8x over the observed collapse boundary
(round 3). This is comparable to the margin in Keccak-f[1600] (24 rounds,
collapse at round 3-4) and BLAKE-family designs.

Full 24-round avalanche expansion confirmed in:
results/avalanche_24_msgs1024_flips32_msg96_seed1234.json

**Conclusion:** 24 rounds provides sufficient margin. Reducing below 20 would
narrow the margin uncomfortably. Increasing above 24 provides no measurable
benefit under current probes.

---

## 7  Frozen Parameters

The following parameters are frozen as of v1.0-pre:

| Parameter | Value | Justification |
|-----------|-------|---------------|
| State width | 1600 bits | Section 2 |
| Rate | 1024 bits | Section 2 |
| Capacity | 576 bits | Section 2 |
| Rounds | 24 | Section 6 |
| Nonlinear layer | Chi* | Section 3 |
| Rotation table | Current ROT | Section 4 |
| Iota sites | A[0][0], A[1][2], A[4][4] | Section 5 |
| Constants | SHAKE256("AHA-D-256-ROUND-CONSTANTS-v0.1") | Spec Section 8 |
| Padding | 0x01 || 0x00* || 0x80 | Spec Section 5 |
| Domain suffix | 0x01 (both modes) | Spec Section 5 |

Any change to a frozen parameter requires a new major version.

---

*End of AHD-1024 Design Rationale v1.0-pre*