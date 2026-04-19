# AHD-1024 (AHA-D-256 v0.2)

AHD-1024 is a candidate cryptographic hash construction built around a 1600-bit sponge permutation with a custom round function, asymmetric constant injection, deterministic constant derivation, and an expanding empirical attack harness.

This repository contains:

* a **reference Rust implementation**
* **deterministic constant derivation**
* **cross-checked test vectors (Python ↔ Rust)**
* **avalanche, reduced-round, ANF, fixed-point, low-weight, rotation, and cube-style parity probes**

The aim is not to claim security by assertion. The aim is to define a candidate, implement it exactly, and subject it to reproducible attack-oriented measurement.

---

## Core Parameters

| Parameter  | Value          |
| ---------- | -------------- |
| State size | 1600 bits      |
| Rate       | 1024 bits      |
| Capacity   | 576 bits       |
| Lanes      | 5 × 5 × 64-bit |
| Rounds     | 24             |
| Digest     | 256 bits       |

---

## Round Structure

Each round is:

Θ → Π → Ρ → Χ → Ι

* **Θ (Theta)**: column parity diffusion with multi-rotation coupling
* **Π (Pi)**: bijective lane permutation
* **Ρ (Rho)**: lane-wise rotations from a fixed offset table
* **Χ (Chi\*)**: extended nonlinear row function
* **Ι (Iota)**: asymmetric injection of three round constants

---

## Constant Derivation

Constants are not arbitrarily chosen.

They are derived deterministically from SHAKE256:

Seed = "AHA-D-256-ROUND-CONSTANTS-v0.1"  
Material = SHAKE256(Seed)

Parsed into 72 little-endian 64-bit values grouped into K0, K1, K2.

---

## Build

cargo build --release

---

## Basic Verification

cargo run --release -- vectors  
cargo run --release -- cross-check

---

## Analysis Harness

### Reduced-round differential search

cargo run --release -- reduced-search [pairs] [msg_len] [seed]  
cargo run --release -- reduced-search-shifted [pairs] [msg_len] [seed]

---

### Avalanche analysis

cargo run --release -- avalanche [n_msgs] [flips_per_msg] [msg_len] [seed]

---

### Exact ANF subspace experiments

cargo run --release -- anf-small [lane_width] [rounds] [tracked_outputs]

---

### Rotation symmetry screening

cargo run --release -- rotation-test [samples] [msg_len] [seed]

---

### Fixed-point search

cargo run --release -- fixed-point [samples] [seed]

---

### Low-weight differential search

cargo run --release -- low-weight [pairs] [msg_len] [seed]

---

### Cube parity probe

cargo run --release -- cube [samples] [msg_len] [cube_bits] [seed]

---

## Current Internal Cryptanalytic Picture

Rounds 1–3 show shallow structure under multiple probes.  
Round 4 is the first point where current attacks stop exposing cheap structure.  
Rounds 5–6 remain stable under current harness.

---

## Extended Cryptanalytic Results

These are exact or statistically bounded results from internal analysis harnesses.

### ANF Exact Subspace Analysis

Projected subspace, exact enumeration (not sampling). Values shown are average algebraic degree.

| Variables | Round 1 | Round 2 | Round 3 | Round 4 | Round 5 | Round 6 |
|-----------|---------|---------|---------|---------|---------|---------|
| 4         | 0.75    | 2.0     | 3.25    | 2.75    | 3.5     | 3.75    |
| 8         | 0.5     | 2.25    | 6.0     | 7.25    | 8.0     | 7.5     |
| 16        | 1.0     | 3.0     | 8.0     | 15.5    | 15.5    | 15.5    |

**Finding:** In the exact 16-variable subspace, degree reaches near-maximum by round 4.

### Rotation Symmetry Screening

Byte-rotation artifact screen across rounds 1–6.

| Rounds | Nontrivial Survivals |
|--------|---------------------|
| 1–6    | 0                   |

**Finding:** No nontrivial byte-rotation matches observed in tested sample.

### Fixed Point Search

State-level fixed-point check: `permute(S, r) == S`

| Rounds | Samples | Fixed Points |
|--------|---------|--------------|
| 1–6    | 200,000 | 0            |

**Finding:** No sampled fixed points through 6 rounds.

### Short Cycle Search

Cycle detection for lengths 2, 3, and 4.

| Cycle Length | Samples | Cycles Found |
|--------------|---------|--------------|
| 2            | 200,000 | 0            |
| 3            | 200,000 | 0            |
| 4            | 200,000 | 0            |

**Finding:** No sampled 2-, 3-, or 4-cycles through 6 rounds.

### Cumulative Security Picture

| Metric                       | Round 3      | Round 4      | Rounds 5–6   |
|------------------------------|--------------|--------------|--------------|
| Differential (single-bit avg) | ~128 (ideal) | ~128 (ideal) | ~128 (ideal) |
| ANF 16-var degree             | 8.0 / 16     | 15.5 / 16    | 15.5 / 16    |
| Rotation survivals            | 0            | 0            | 0            |
| Fixed points (200k)           | 0            | 0            | 0            |

**Conclusion:** Round 3 achieves statistical ideality in differential metrics. Round 4 achieves near-maximum algebraic degree. No structural artifacts detected in any metric through round 6.

## Philosophy

All claims are tied to code and measurable outputs. No assumptions.

---

## License

MUI


## Status

AHD-1024 has reached a critical reproducibility milestone.

### Specification

- Endianness and padding normalization: ✅ Complete
- Canonical mapping (bytes → lanes → state): ✅ Defined
- Domain separation scheme: ✅ Defined
- Padding rule (0x01 || 0x00* || 0x80): ✅ Frozen

See:
- spec/endianness-padding-normalization.md

---

### Independent Implementations

Three independent implementations now exist and agree exactly on all frozen test vectors:

| Language | Status | Notes |
|----------|--------|-------|
| Rust     | ✅ Reference | Primary implementation |
| Python   | ✅ Independent | Clean, readable, matches spec exactly |
| C        | ✅ Independent | Minimal, portable, constant-embedded |

All three implementations produce **bit-identical outputs** for:
- Hash mode (32 bytes)
- XOF mode (64 bytes)
- All canonical test vectors

---

### Verified Test Coverage

The following inputs are verified across all implementations:
- "" (empty)
- "a"
- "abc"
- "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" (52 bytes)
- 0x00 × 126 bytes (boundary case)
- 0x00 × 128 bytes (full block)
- 0xff × 128 bytes (max entropy block)

Artifacts:
- spec/test-vectors/hash-and-xof-prefreeze.json
- spec/constants/round-constants-prefreeze.json

Python verification:
    python3 impl/python/test_vectors.py
    Expected output: ALL_OK

C verification:
    cc -O2 -std=c11 -Iimpl/c impl/c/ahd1024.c impl/c/test_vectors.c -o test && ./test
    Expected output: ALL_OK

---

### What This Means

AHD-1024 is now:
- **Unambiguously specified**
- **Independently implementable**
- **Deterministic across implementations**
- **Fully reproducible from spec artifacts**

This satisfies the core requirement for a cryptographic candidate:
  Three independent implementations derived from the specification produce identical outputs.

---

### Next Phase

Phase 3: Extended Empirical Cryptanalysis
- Full 24-round avalanche matrix
- Linear trail search (MILP/SAT)
- Higher-order differential probes
- Bias analysis across all output bits
- Lane activity propagation metrics

---

### Version

Current state: **v1.0-pre (post-normalization, pre-freeze)**

Any change to:
- padding
- endianness
- constants
- round structure

will require a **new major version**

---

### Summary

AHD-1024 has transitioned from:
  Working design

to:
  **Reproducible cryptographic candidate with independent verification**

