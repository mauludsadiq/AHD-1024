# AHD-1024 (AHA-D-256 v0.2)

AHD-1024 is a candidate cryptographic hash construction based on a 1600-bit sponge permutation with a custom round function, asymmetric constant injection, and a full empirical analysis harness.

This repository contains:

* A **reference Rust implementation**
* **Deterministic constant derivation**
* **Cross-checked test vectors (Python ↔ Rust)**
* **Avalanche, differential, ANF, and symmetry analysis tools**

The goal is not “yet another hash,” but a **fully inspectable, reproducible cryptographic candidate** with measurable properties.

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

## Design Structure

Each round:

Θ → Π → Ρ → Χ → Ι

* **Θ (Theta)**: column parity diffusion with multi-rotation mixing
* **Π (Pi)**: bijective lane permutation
* **Ρ (Rho)**: lane rotations (non-symmetric table)
* **Χ (Chi*)**: nonlinear layer (extended vs Keccak-style)
* **Ι (Iota)**: asymmetric constant injection (3 lanes per round)

---

## Constant Derivation

Constants are **not hardcoded**.

They are derived deterministically:

```
Seed = "AHA-D-256-ROUND-CONSTANTS-v0.1"
K = SHAKE256(Seed)
```

→ parsed into 72 × 64-bit values

This guarantees:

* reproducibility
* no hidden structure
* no “nothing-up-my-sleeve” ambiguity

---

## Implementation

### Build

```bash
cargo build --release
```

### Run hash / vectors

```bash
cargo run --release -- vectors
cargo run --release -- cross-check
```

---

## Analysis Harness

### Reduced-round differential search

```bash
cargo run --release -- reduced-search [pairs] [msg_len] [seed]
cargo run --release -- reduced-search-shifted [pairs] [msg_len] [seed]
```

Measures:

* output difference uniqueness
* collision patterns
* diffusion quality

---

### Avalanche testing

```bash
cargo run --release -- avalanche [n_msgs] [flips_per_msg] [msg_len] [seed]
```

Example result:

* avg flip rate ≈ **0.5001**
* max deviation ≈ **0.0028**

This is near-ideal.

---

### Exact ANF (Algebraic Degree)

```bash
cargo run --release -- anf-small [lane_width] [rounds] [tracked_outputs]
```

Constraints:

* exact truth-table expansion ≤ 16 variables

Observed:

| lane width | round | degree           |
| ---------- | ----- | ---------------- |
| 4          | 4     | ~16 (saturation) |

This confirms **rapid algebraic degree growth**, not just theoretical bounds.

---

### Rotation Symmetry Test

```bash
cargo run --release -- rotation-test [samples] [msg_len] [seed]
```

Tests for:

> f(rot(x)) = rot(f(x))

Result (after fixing byte-alignment artifact):

* **0 surviving rotational matches (rounds 1–6)**

This eliminates simple rotational invariants.

---

## Current Empirical Results

* Strong avalanche behavior (~0.5)
* No repeated output differences in large random sampling
* Rapid ANF degree growth (saturates small domains)
* No observed rotational symmetry (nontrivial)
* No structural distinction between baseline and shifted rotation table under random testing

---

## Repository Structure

```
AHD-1024/
├── Cargo.toml
├── src/
│   ├── lib.rs        # core permutation + hash
│   └── main.rs       # CLI + analysis tools
├── python_vectors/
│   └── results_v0_2.json
├── results/          # generated experiment outputs
├── README.md
└── .gitignore
```

---

## Status

**Version:** v0.2
**State:** Candidate under internal cryptanalysis

This is not production-ready cryptography.

It is:

* a **structured candidate**
* with **measurable properties**
* and a **reproducible analysis pipeline**

---

## Next Steps

Planned directions:

* Fixed-point search
* Low-weight differential exploration
* Division property / cube analysis
* Lane-level rotational invariance checks
* Formal spec extraction (v1.0 candidate)

---

## Philosophy

This project treats cryptography as:

> something to be *measured, reproduced, and broken* — not assumed.

Every claim here is:

* backed by code
* reproducible
* falsifiable

---

## License

MUI
