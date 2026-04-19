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

## Status

Version: v0.2  
State: candidate under active internal cryptanalysis

---

## Philosophy

All claims are tied to code and measurable outputs. No assumptions.

---

## License

MUI

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

