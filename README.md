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
