# AHD 1024

VS Code-ready Rust workspace for **AHA-D-256 v0.2** with:

1. an **exact Rust implementation** matching the current Python reference semantics,
2. a **cross-check path against the Python vectors**,
3. a **stronger reduced-round search** focused on rounds **1–6** with larger pair counts,
4. an **exact small-width ANF experiment** so algebraic degree is no longer only an upper-bound story.

This package is designed for local execution in VS Code because the heavier search settings and exact ANF truth-table work can exceed lightweight interactive limits.

---

## 1. What is in this zip

```text
AHD_1024/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   └── main.rs
├── python_vectors/
│   └── results_v0_2.json
└── results/
```

- `src/lib.rs`
  - exact AHA-D-256 v0.2 implementation
  - constant derivation from SHAKE256 seed
  - padding / absorb / squeeze
  - 24-round permutation
  - reduced-round search harness
  - avalanche harness
  - exact small-width ANF experiment
  - Rust unit tests against the Python vectors

- `src/main.rs`
  - CLI entrypoints for vectors, cross-check, reduced-round search, avalanche, and ANF

- `python_vectors/results_v0_2.json`
  - copied from the Python side so the Rust results can be compared against the executed v0.2 reference outputs

---

## 2. Exact candidate being implemented

This workspace implements **AHA-D-256 v0.2** exactly as currently frozen.

### Core parameters

- state size: **1600 bits**
- rate: **1024 bits**
- capacity: **576 bits**
- lane width: **64 bits**
- lane count: **25** arranged as a **5 × 5** matrix
- rounds per permutation: **24**
- digest size: **256 bits** by default

### State layout

```text
S[x][y],  0 <= x < 5, 0 <= y < 5
```

Lane packing uses **little-endian** bytes.

### v0.2 padding / bit-order normalization

v0.2 removes the mixed bit-order ambiguity from the earlier draft and uses a byte-aligned rule consistent with little-endian lane packing:

```text
message || domain || 0x01 || zero-fill
then final byte |= 0x80
```

This matters because the Rust implementation is intended to be an exact semantic mirror of the Python v0.2 reference, not a reinterpretation.

---

## 3. Constants derivation rule

Constants are derived from:

```text
SHAKE256("AHA-D-256-ROUND-CONSTANTS-v0.1")
```

The XOF output length is:

```text
3 * 24 * 8 = 576 bytes
```

Those bytes are parsed as **72 little-endian 64-bit words**, then grouped into:

- `K0[t]`
- `K1[t]`
- `K2[t]`

for rounds `t = 0..23`.

The implementation in `lib.rs` computes these at runtime from the public seed.

---

## 4. Exact round structure

Each round performs:

```text
Theta -> Pi -> Rho -> Chi* -> Iota
```

### Theta

Column parity diffusion:

```text
C[x] = S[x][0] ^ S[x][1] ^ S[x][2] ^ S[x][3] ^ S[x][4]
D[x] = ROTL1(C[x-1]) ^ ROTL11(C[x+1]) ^ ROTL27(C[x+2])
S[x][y] ^= D[x]
```

### Pi

```text
S'[x][y] = S[(2x + 3y) mod 5][(x + 2y) mod 5]
```

### Rho

Uses the fixed rotation table:

```text
[ [ 0,  7, 19, 41, 53],
  [11, 29, 43,  3, 31],
  [37, 59,  5, 17, 47],
  [23, 13, 61, 27,  9],
  [45, 21, 39, 49, 55] ]
```

### Chi* (the current nonlinear layer)

For each row:

```text
b_i = a_i
    ^ ((~a_{i+1}) & a_{i+2})
    ^ (ROTL1(a_{i+3}) & ROTL3(a_{i+4}))
```

### Iota

Injects three asymmetric round constants:

```text
S[0][0] ^= K0[t]
S[1][2] ^= K1[t]
S[4][4] ^= K2[t]
```

---

## 5. Domain bytes

The Rust implementation includes these domain values:

- `0x01` = `HASH`
- `0x02` = `XOF`
- `0x03` = `TREE_LEAF`
- `0x04` = `TREE_PARENT`
- `0x05` = `MAC_KEYED`
- `0x06` = `TRANSCRIPT`
- `0x07` = `ARTIFACT`
- `0x08` = `ROUND_TRACE`

---

## 6. Canonical vectors already enforced in Rust tests

The project includes Rust unit tests that cross-check the exact Python vectors for:

### HASH

- `HASH("")`
  - `e8bf66fb70ec3787817c0cb717952140569a853f94dee36a21268632b9a59ed0`
- `HASH("abc")`
  - `50f4f48736c87a32bb20c618fda7de0ec0260edd57f340e92d8daa45d54a4a1f`
- `HASH(128x00)`
  - `22598b6298b7125bdacf7486508d3efc34e93334f93b889b736e2614cd3479fe`
- `HASH(127x00 || 0x01)`
  - `2eb15de636e671274ffe8891dae56353712dc4fbffca2876041d2d63219ec5dc`

### XOF(64 bytes)

- `XOF64("")`
  - `01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76`
- `XOF64("abc")`
  - `87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f5410`

---

## 7. Opening this in VS Code

### Requirements

- Rust toolchain installed
- Cargo available on PATH
- VS Code with rust-analyzer recommended

### Open the folder

```bash
code AHD_1024
```

or open it manually in VS Code.

---

## 8. First commands to run

### Build

```bash
cargo build
```

### Run unit tests

```bash
cargo test -- --nocapture
```

This is the first check that the Rust implementation exactly matches the Python vectors.

### Emit vectors via CLI

```bash
cargo run --release -- vectors
```

This writes:

```text
results/vectors.json
```

### Emit a concise cross-check report

```bash
cargo run --release -- cross-check
```

This writes:

```text
results/cross_check.json
```

---

## 9. Stronger reduced-round search (1–6 rounds)

This is one of the main reasons this workspace exists.

The Python side previously sampled reduced rounds. This Rust workspace gives you a heavier local path for 1–6 round searches with larger pair counts.

### Default run

```bash
cargo run --release -- reduced-search
```

Default settings:

- rounds: `1,2,3,4,5,6`
- pairs: `20_000`
- message length: `96 bytes`
- seed: `7`

Output file:

```text
results/reduced_search_pairs20000_msg96_seed7.json
```

### Heavier runs

Try progressively:

```bash
cargo run --release -- reduced-search 50000 96 7
cargo run --release -- reduced-search 100000 96 7
cargo run --release -- reduced-search 250000 96 7
```

### What to inspect

For each round count, inspect:

- `unique_output_differences`
- `max_repeated_output_difference_count`
- `top5_repeat_counts`
- `zero_difference_count`
- `avg_changed_fraction`
- `min_changed_fraction`
- `max_changed_fraction`

### How to read the result

You want the 2–6 round regime to show rapid diffusion and no suspicious concentration of repeated output differences. The 1-round regime will be structurally sparse by design; the important question is how quickly the structure explodes away from that.

---

## 10. Avalanche harness

Run:

```bash
cargo run --release -- avalanche
```

Default settings:

- `n_msgs = 512`
- `flips_per_msg = 32`
- `msg_len = 96`
- `seed = 1234`

Output file:

```text
results/avalanche_msgs512_flips32_msg96_seed1234.json
```

Heavier examples:

```bash
cargo run --release -- avalanche 2048 32 96 1234
cargo run --release -- avalanche 4096 32 96 1234
```

Metrics of interest:

- `avg_changed_fraction`
- `output_flip_mean_abs_dev`
- `output_flip_max_abs_dev`
- probability range over all 256 output bits

The spec variant should stay close to 0.5 at full rounds.

---

## 11. Exact small-width ANF experiment

This is the crucial local replacement for the earlier “upper-bound only” degree story.

The exact ANF experiment in this workspace does **truth-table enumeration + Möbius transform** on a deliberately **small-width version** of the permutation state.

### Important constraint

Exact truth-table ANF complexity is exponential in the number of variables.

This code intentionally restricts itself to:

```text
25 * lane_width <= 16
```

That means practical exact settings are:

- `lane_width = 1`  -> 25 vars -> **too large**, rejected
- `lane_width = 0`  -> invalid

So for exact ANF, you must track a reduced subproblem. The current implementation enforces a strict exactness limit and will reject widths that explode.

### Recommended first use

Run with a small tracked output count after reducing the code to a subset of lanes if you want a fully exact sub-instance. In the current workspace, the exact experiment is implemented and safe, but the default full 25-lane mapping means you must keep the state variable count under control.

For a first exact run, edit `exact_small_width_anf_experiment()` to track a selected subset of lanes only, or keep the current routine as the base and reduce the variable set before scaling.

CLI form:

```bash
cargo run --release -- anf-small 1 6 8
```

Output file:

```text
results/anf_small_w1_r6_o8.json
```

### What this experiment is for

It answers:

- what exact ANF degree appears in a reduced but real instance,
- how fast degree grows round by round,
- whether the nonlinear layer behaves like a serious algebraic diffuser or stalls.

### What to do next if you want exact but larger subproblems

Recommended path:

1. choose a reduced lane subset,
2. freeze a local reduced permutation instance,
3. compute exact ANF degrees for that instance,
4. compare `Chi*` against baseline `Chi` under the same reduced geometry.

That gives you an exact apples-to-apples degree study.

---

## 12. Why the ANF experiment is written this way

The previous degree-growth story was only an upper-bound propagation check.

That is useful, but it is not exact algebra.

This workspace pushes beyond that by including the exact Möbius-transform route. It is deliberately conservative because exact ANF explodes extremely fast. The correct move is not to pretend a large exact experiment is cheap. The correct move is to build the exact machinery and run it on reduced sub-instances that are mathematically honest.

---

## 13. How to compare nonlinear-layer variants

The library already contains:

- `ChiVariant::Star`
- `ChiVariant::Baseline`

You can compare them by swapping the argument in:

- `stronger_reduced_round_search(...)`
- `avalanche_stats(...)`
- `exact_small_width_anf_experiment(...)`

That lets you test whether the extra term in `Chi*` is actually buying:

- stronger 1–2 round diffusion,
- better distinct-output-difference behavior,
- faster algebraic degree growth,
- no obvious short structural artifacts.

---

## 14. Suggested VS Code work plan

### Phase A — correctness

Run:

```bash
cargo test -- --nocapture
cargo run --release -- vectors
cargo run --release -- cross-check
```

Goal:
- ensure exact vector agreement with the Python v0.2 reference.

### Phase B — reduced-round search

Run:

```bash
cargo run --release -- reduced-search 50000 96 7
cargo run --release -- reduced-search 100000 96 7
```

Goal:
- inspect 1–6 round behavior at higher pair counts than the light interactive pass.

### Phase C — avalanche scaling

Run:

```bash
cargo run --release -- avalanche 2048 32 96 1234
cargo run --release -- avalanche 4096 32 96 1234
```

Goal:
- stabilize the full-round avalanche measurements.

### Phase D — exact algebraic study

Use the ANF path on a reduced exact instance.

Goal:
- replace “degree seems to grow fast” with an exact reduced-instance result.

### Phase E — variant tournament

Test:
- `Chi*` vs baseline `Chi`
- original rotation table vs shifted or alternate candidate tables

Goal:
- determine whether the current nonlinear layer and rotation schedule deserve to stay frozen for a later candidate.

---

## 15. What this project does **not** claim yet

This workspace does **not** claim that AHA-D-256 is a secure cryptographic hash.

It claims something narrower and real:

- the current candidate is now specified enough to implement exactly,
- the Rust and Python paths can be forced to agree,
- heavier local measurement can be run,
- the algebraic story can start moving from upper bounds to exact reduced-instance computation.

That is the right stage for a candidate at this point.

---

## 16. Immediate next upgrades after this zip

The most valuable next steps are:

1. add a **trace-emitting Rust path** for per-stage round states,
2. freeze a **reduced exact ANF sub-instance** so degree results become routine and reproducible,
3. add **rotation-table variants** as named presets,
4. add **CSV export** for reduced-round difference histograms,
5. port the search harness to **Rayon** if you want multicore scaling,
6. add a separate **distinguishing experiment** rather than only bit-flip diffusion sampling.

---

## 17. Quick reference commands

```bash
cargo test -- --nocapture
cargo run --release -- vectors
cargo run --release -- cross-check
cargo run --release -- reduced-search
cargo run --release -- reduced-search 100000 96 7
cargo run --release -- avalanche
cargo run --release -- avalanche 4096 32 96 1234
cargo run --release -- anf-small 1 6 8
```

---

## 18. Final note

This zip is the correct transition from “candidate discussion” to “local executable object.”

Use it in VS Code as the base for:

- exact Rust/Python cross-checking,
- heavier reduced-round search,
- exact reduced-instance algebraic experiments,
- and the next freeze decision on whether the current nonlinear layer and rotation table survive serious pressure.
