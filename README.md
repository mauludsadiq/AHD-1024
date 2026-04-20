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

### First Phase 3 Result: Avalanche Matrix Screening

A first avalanche-matrix campaign has now been completed.

Measured runs:
- `64 × 256` matrix at `1024` messages per input bit
- `64 × 256` matrix at `4096` messages per input bit
- `128 × 256` matrix at `4096` messages per input bit

Key results from the higher-confidence runs:

- `64 × 256`, `4096` samples/input bit  
  - `global_min_prob = 0.464111328125`  
  - `global_max_prob = 0.531494140625`  
  - `global_mean_abs_dev = 0.0062551796436309814`  
  - `global_max_abs_dev = 0.035888671875`

- `128 × 256`, `4096` samples/input bit  
  - `global_min_prob = 0.464111328125`  
  - `global_max_prob = 0.531494140625`  
  - `global_mean_abs_dev = 0.006231091916561127`  
  - `global_max_abs_dev = 0.035888671875`

Interpretation:

- Increasing sample count from `1024` to `4096` reduced deviation substantially, consistent with sampling-noise shrinkage.
- Expanding coverage from `64` to `128` input bits did not reveal a hidden weak band.
- The first `128` message bits exhibit broadly healthy avalanche behavior against all `256` output bits at this resolution.

Artifacts:
- `results/avalanche_matrix_in64_msgs1024_msg96_seed1234.json`
- `results/avalanche_matrix_in64_msgs4096_msg96_seed1234.json`
- `results/avalanche_matrix_in128_msgs4096_msg96_seed1234.json`


### Phase 3 Result: Linear / Correlation Matrix Screening

A full linear correlation matrix scan has now been completed across early rounds.

Measured runs:
- `64 × 256` at `4096` samples per input bit
- `64 × 256` at `16384` samples per input bit
- `128 × 256` at `16384` samples per input bit

Key high-confidence results (`16384` samples/input bit):

- Round global mean bias:
  - ~`0.00310 – 0.00313` across rounds `1–6`

- Round global max bias:
  - ~`0.0157 – 0.0169` across rounds `1–6`

Interpretation:

- Increasing sample count from `4096` → `16384` reduced previously observed maxima, consistent with sampling-noise collapse.
- Expanding coverage from `64` → `128` input bits did not reveal any structured bias band.
- Mean bias aligns with expected statistical noise:
  - σ ≈ `sqrt(0.25 / 16384) = 0.00390625`
- Observed maxima are consistent with extreme-value effects over a large matrix, not structural leakage.

Conclusion:

- No persistent linear/correlation weakness is observed in the first `128` input bits across rounds `1–6` at current resolution.

Artifacts:
- `results/linear_matrix_in64_out256_samples4096_msg96_seed1234.json`
- `results/linear_matrix_in64_out256_samples16384_msg96_seed1234.json`
- `results/linear_matrix_in128_out256_samples16384_msg96_seed1234.json`


### Phase 3 Result: Higher-Order Differential Screening

Higher-order differential scans have now been added for orders `2` and `3`.

Measured runs:
- `order = 2`, `50000` pairs, `msg_len = 96`
- `order = 3`, `50000` pairs, `msg_len = 96`

Key results:

#### Order 2
- Round `1`:
  - `avg_changed_bits = 14.77528`
  - fully low-weight, as expected
- Round `2`:
  - `avg_changed_bits = 125.46526`
  - residual low-weight tail remains:
    - `count_le_32 = 19`
    - `count_le_48 = 69`
    - `count_le_64 = 88`
- Rounds `3–6`:
  - `avg_changed_bits ≈ 128`
  - `min_changed_bits >= 95`
  - `count_le_32 = count_le_48 = count_le_64 = 0`

#### Order 3
- Round `1`:
  - `avg_changed_bits = 21.45112`
  - still shallow, as expected
- Round `2`:
  - `avg_changed_bits = 127.24164`
  - `min_changed_bits = 93`
  - `count_le_32 = count_le_48 = count_le_64 = 0`
- Rounds `3–6`:
  - `avg_changed_bits ≈ 128`
  - `min_changed_bits >= 90`
  - `count_le_32 = count_le_48 = count_le_64 = 0`

Interpretation:

- Order-2 differentials retain a very small round-2 low-weight tail, but that tail disappears by round `3`.
- Order-3 differentials are already fully mixed by round `2` at this resolution.
- Across rounds `3–6`, both orders show no low-weight residual at the tested thresholds.

Artifacts:
- `results/higher_order_o2_pairs50000_msg96_seed1234.json`
- `results/higher_order_o3_pairs50000_msg96_seed1234.json`
- `results/higher_order_o4_pairs50000_msg96_seed1234.json`

#### Order 4
- Round `1`:
  - `avg_changed_bits = 27.73712`
  - still shallow, as expected
- Round `2`:
  - `avg_changed_bits = 127.6997`
  - `min_changed_bits = 96`
  - `count_le_32 = count_le_48 = count_le_64 = 0`
- Rounds `3–6`:
  - `avg_changed_bits ≈ 128`
  - `min_changed_bits >= 91`
  - `count_le_32 = count_le_48 = count_le_64 = 0`

Updated interpretation:

- Order-2 differentials retain a very small round-2 low-weight tail, but that tail disappears by round `3`.
- Order-3 differentials are already fully mixed by round `2` at this resolution.
- Order-4 differentials are also fully mixed by round `2` at this resolution.
- Across rounds `3–6`, all tested higher orders show no low-weight residual at the tested thresholds.


### Phase 3 Result: Lane Activity Propagation

A lane-activity propagation probe has now been completed using `50000` samples at `msg_len = 96`.

Measured run:
- `results/lane_activity_samples50000_msg96_seed1234.json`

Key results:
- Round `1`:
  - `active_output_bits_avg = 7.6146`
  - `active_output_bits_min = 2`
  - `active_output_bits_max = 15`
- Round `2`:
  - `active_output_bits_avg = 115.53068`
  - `active_output_bits_min = 75`
  - `active_output_bits_max = 157`
- Round `3`:
  - `active_output_bits_avg = 128.09118`
  - `active_output_bits_min = 94`
  - `active_output_bits_max = 158`
- Round `4`:
  - `active_output_bits_avg = 127.90678`
  - `active_output_bits_min = 92`
  - `active_output_bits_max = 162`
- Round `5`:
  - `active_output_bits_avg = 128.0262`
  - `active_output_bits_min = 95`
  - `active_output_bits_max = 162`
- Round `6`:
  - `active_output_bits_avg = 127.99412`
  - `active_output_bits_min = 95`
  - `active_output_bits_max = 161`

Interpretation:
- Round `2` already shows strong expansion from a single input-bit flip.
- Round `3` is effectively saturated.
- Rounds `3–6` remain stable with no sign of weak propagation bands.


### Phase 3 Result: Short-Cycle / Quartet-Connectivity Screening

Short-cycle connectivity screens have been rerun at `200000` samples across rounds `1–6`.

Measured runs:
- `results/two_cycle_samples200000_seed7.json`
- `results/three_cycle_samples200000_seed7.json`
- `results/four_cycle_samples200000_seed7.json`

Results:
- `two-cycle`: all zero across rounds `1–6`
- `three-cycle`: all zero across rounds `1–6`
- `four-cycle`: all zero across rounds `1–6`

Interpretation:
- No immediate short-cycle or quartet-style connectivity artifact is visible under this screen.
- This does not prove absence of deeper boomerang structure, but it removes the most obvious reduced-round cycle signal at the tested depth.


### Next Phase

Phase 3: Extended Empirical Cryptanalysis
- SAT/MILP-style reduced-round structure search
- Full 24-round avalanche expansion
- Additional higher-order differential orders as needed

Decision:
- Next strongest move: **SAT/MILP-style reduced-round structure search**
- Reason: avalanche, linear, higher-order, lane-activity, and short-cycle/quartet connectivity screens are all currently favorable.

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

## Philosophy

All claims are tied to code and measurable outputs. No assumptions.

---

## License

MUI
