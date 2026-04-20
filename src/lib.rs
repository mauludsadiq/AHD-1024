use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use sha3::{digest::{ExtendableOutput, Update, XofReader}, Shake256};
use std::collections::{HashMap, HashSet};

pub const SEED: &[u8] = b"AHA-D-256-ROUND-CONSTANTS-v0.1";
pub const MASK64: u64 = u64::MAX;
pub const RATE_BITS: usize = 1024;
pub const RATE_BYTES: usize = RATE_BITS / 8;
pub const STATE_LANES: usize = 25;
pub const ROUNDS: usize = 24;

pub const ROT: [[u32; 5]; 5] = [
    [0, 7, 19, 41, 53],
    [11, 29, 43, 3, 31],
    [37, 59, 5, 17, 47],
    [23, 13, 61, 27, 9],
    [45, 21, 39, 49, 55],
];

pub type State = [[u64; 5]; 5];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    Hash = 0x01,
    Xof = 0x02,
    TreeLeaf = 0x03,
    TreeParent = 0x04,
    MacKeyed = 0x05,
    Transcript = 0x06,
    Artifact = 0x07,
    RoundTrace = 0x08,
}

impl Domain {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "HASH" => Some(Self::Hash),
            "XOF" => Some(Self::Xof),
            "TREE_LEAF" => Some(Self::TreeLeaf),
            "TREE_PARENT" => Some(Self::TreeParent),
            "MAC_KEYED" => Some(Self::MacKeyed),
            "TRANSCRIPT" => Some(Self::Transcript),
            "ARTIFACT" => Some(Self::Artifact),
            "ROUND_TRACE" => Some(Self::RoundTrace),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constants {
    pub k0: [u64; ROUNDS],
    pub k1: [u64; ROUNDS],
    pub k2: [u64; ROUNDS],
}

pub fn derive_constants() -> Constants {
    let mut hasher = Shake256::default();
    hasher.update(SEED);
    let mut xof = hasher.finalize_xof();
    let mut material = [0u8; 3 * ROUNDS * 8];
    xof.read(&mut material);
    let mut words = [0u64; 3 * ROUNDS];
    for (i, chunk) in material.chunks_exact(8).enumerate() {
        words[i] = u64::from_le_bytes(chunk.try_into().unwrap());
    }
    let mut k0 = [0u64; ROUNDS];
    let mut k1 = [0u64; ROUNDS];
    let mut k2 = [0u64; ROUNDS];
    for t in 0..ROUNDS {
        k0[t] = words[3 * t];
        k1[t] = words[3 * t + 1];
        k2[t] = words[3 * t + 2];
    }
    Constants { k0, k1, k2 }
}

#[inline]
pub fn rotl64(x: u64, n: u32) -> u64 {
    x.rotate_left(n)
}

pub fn blank_state() -> State {
    [[0u64; 5]; 5]
}

pub fn theta(s: &State) -> State {
    let mut c = [0u64; 5];
    for x in 0..5 {
        c[x] = s[x][0] ^ s[x][1] ^ s[x][2] ^ s[x][3] ^ s[x][4];
    }
    let mut d = [0u64; 5];
    for x in 0..5 {
        d[x] = rotl64(c[(x + 4) % 5], 1) ^ rotl64(c[(x + 1) % 5], 11) ^ rotl64(c[(x + 2) % 5], 27);
    }
    let mut out = [[0u64; 5]; 5];
    for x in 0..5 {
        for y in 0..5 {
            out[x][y] = s[x][y] ^ d[x];
        }
    }
    out
}

pub fn pi_stage(s: &State) -> State {
    let mut out = [[0u64; 5]; 5];
    for x in 0..5 {
        for y in 0..5 {
            out[x][y] = s[(2 * x + 3 * y) % 5][(x + 2 * y) % 5];
        }
    }
    out
}

pub fn rho(s: &State, rot: &[[u32; 5]; 5]) -> State {
    let mut out = [[0u64; 5]; 5];
    for x in 0..5 {
        for y in 0..5 {
            out[x][y] = rotl64(s[x][y], rot[x][y]);
        }
    }
    out
}

pub fn chi_star(s: &State) -> State {
    let mut out = [[0u64; 5]; 5];
    for y in 0..5 {
        let a = [s[0][y], s[1][y], s[2][y], s[3][y], s[4][y]];
        for i in 0..5 {
            out[i][y] = a[i]
                ^ ((!a[(i + 1) % 5]) & a[(i + 2) % 5])
                ^ (rotl64(a[(i + 3) % 5], 1) & rotl64(a[(i + 4) % 5], 3));
        }
    }
    out
}

pub fn chi_baseline(s: &State) -> State {
    let mut out = [[0u64; 5]; 5];
    for y in 0..5 {
        let a = [s[0][y], s[1][y], s[2][y], s[3][y], s[4][y]];
        for i in 0..5 {
            out[i][y] = a[i] ^ ((!a[(i + 1) % 5]) & a[(i + 2) % 5]);
        }
    }
    out
}

pub fn iota(s: &State, t: usize, constants: &Constants) -> State {
    let mut out = *s;
    out[0][0] ^= constants.k0[t];
    out[1][2] ^= constants.k1[t];
    out[4][4] ^= constants.k2[t];
    out
}

#[derive(Debug, Clone, Copy)]
pub enum ChiVariant {
    Star,
    Baseline,
}

pub fn permute(mut s: State, rounds: usize, constants: &Constants, chi: ChiVariant, rot: &[[u32; 5]; 5]) -> State {
    for t in 0..rounds {
        s = theta(&s);
        s = pi_stage(&s);
        s = rho(&s, rot);
        s = match chi {
            ChiVariant::Star => chi_star(&s),
            ChiVariant::Baseline => chi_baseline(&s),
        };
        s = iota(&s, t, constants);
    }
    s
}

pub fn pad_v02(message: &[u8], domain: Domain) -> Vec<u8> {
    let mut out = Vec::with_capacity(message.len() + 2 + RATE_BYTES);
    out.extend_from_slice(message);
    out.push(domain as u8);
    out.push(0x01);
    while out.len() % RATE_BYTES != RATE_BYTES - 1 {
        out.push(0);
    }
    out.push(0x80);
    out
}

pub fn absorb_blocks(padded: &[u8], rounds: usize, constants: &Constants, chi: ChiVariant, rot: &[[u32; 5]; 5]) -> State {
    let mut s = blank_state();
    for block in padded.chunks_exact(RATE_BYTES) {
        for i in 0..16 {
            let lane = u64::from_le_bytes(block[8 * i..8 * i + 8].try_into().unwrap());
            let x = i % 5;
            let y = i / 5;
            s[x][y] ^= lane;
        }
        s = permute(s, rounds, constants, chi, rot);
    }
    s
}

pub fn squeeze_bytes(mut s: State, out_len: usize, rounds: usize, constants: &Constants, chi: ChiVariant, rot: &[[u32; 5]; 5]) -> Vec<u8> {
    let mut out = Vec::with_capacity(out_len);
    while out.len() < out_len {
        for i in 0..16 {
            let x = i % 5;
            let y = i / 5;
            out.extend_from_slice(&s[x][y].to_le_bytes());
            if out.len() >= out_len {
                out.truncate(out_len);
                return out;
            }
        }
        s = permute(s, rounds, constants, chi, rot);
    }
    out
}

pub fn aha_hash(message: &[u8], domain: Domain, out_len: usize, rounds: usize, constants: &Constants, chi: ChiVariant, rot: &[[u32; 5]; 5]) -> Vec<u8> {
    let padded = pad_v02(message, domain);
    let s = absorb_blocks(&padded, rounds, constants, chi, rot);
    squeeze_bytes(s, out_len, rounds, constants, chi, rot)
}

pub fn hex_of(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoundStats {
    pub pairs: usize,
    pub unique_output_differences: usize,
    pub max_repeated_output_difference_count: usize,
    pub top5_repeat_counts: Vec<usize>,
    pub zero_difference_count: usize,
    pub avg_changed_fraction: f64,
    pub min_changed_fraction: f64,
    pub max_changed_fraction: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReducedRoundReport {
    pub variant: String,
    pub rounds: HashMap<usize, RoundStats>,
}

pub fn popcount_bytes(bytes: &[u8]) -> u32 {
    bytes.iter().map(|b| b.count_ones()).sum()
}

pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b).map(|(x, y)| x ^ y).collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LowWeightRoundStats {
    pub pairs: usize,
    pub min_changed_bits: u32,
    pub avg_changed_bits: f64,
    pub max_changed_bits: u32,
    pub count_le_32: usize,
    pub count_le_48: usize,
    pub count_le_64: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LowWeightReport {
    pub variant: String,
    pub rounds: HashMap<usize, LowWeightRoundStats>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredDifferentialRoundStats {
    pub pairs_per_pattern: usize,
    pub pattern: String,
    pub min_changed_bits: u32,
    pub avg_changed_bits: f64,
    pub max_changed_bits: u32,
    pub count_le_32: usize,
    pub count_le_48: usize,
    pub count_le_64: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredDifferentialReport {
    pub variant: String,
    pub rounds: HashMap<usize, HashMap<String, StructuredDifferentialRoundStats>>,
}

fn apply_structured_diff(m2: &mut [u8], pattern: &str, base_pos: usize, msg_bits: usize) {
    match pattern {
        "single_bit" => {
            let p = base_pos % msg_bits;
            m2[p / 8] ^= 1u8 << (p % 8);
        }
        "adjacent_2" => {
            for off in 0..2 {
                let p = (base_pos + off) % msg_bits;
                m2[p / 8] ^= 1u8 << (p % 8);
            }
        }
        "adjacent_4" => {
            for off in 0..4 {
                let p = (base_pos + off) % msg_bits;
                m2[p / 8] ^= 1u8 << (p % 8);
            }
        }
        "same_byte_full" => {
            let byte = (base_pos / 8) % (msg_bits / 8);
            m2[byte] ^= 0xff;
        }
        "same_lane_8" => {
            for off in 0..8 {
                let p = (base_pos + off * 64) % msg_bits;
                m2[p / 8] ^= 1u8 << (p % 8);
            }
        }
        _ => panic!("unknown structured differential pattern"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinearRoundStats {
    pub samples: usize,
    pub input_bit: usize,
    pub output_bit: usize,
    pub bias: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinearReport {
    pub rounds: HashMap<usize, LinearRoundStats>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LinearMatrixReport {
    pub n_input_bits: usize,
    pub n_output_bits: usize,
    pub samples_per_input_bit: usize,
    pub rounds: HashMap<usize, Vec<Vec<f64>>>,
    pub round_global_max_bias: HashMap<usize, f64>,
    pub round_global_mean_bias: HashMap<usize, f64>,
}

pub fn linear_correlation_matrix(
    rounds_list: &[usize],
    samples_per_input_bit: usize,
    msg_len: usize,
    n_input_bits: usize,
    n_output_bits: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> LinearMatrixReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rounds = HashMap::new();
    let mut round_global_max_bias = HashMap::new();
    let mut round_global_mean_bias = HashMap::new();

    for &r in rounds_list {
        let mut matrix = vec![vec![0.0_f64; n_output_bits]; n_input_bits];
        let mut max_bias = 0.0_f64;
        let mut sum_bias = 0.0_f64;
        let mut cells = 0usize;

        for input_bit in 0..n_input_bits {
            let mut equal_counts = vec![0usize; n_output_bits];

            for _ in 0..samples_per_input_bit {
                let mut m = vec![0u8; msg_len];
                rng.fill(&mut m[..]);
                let in_bit = (m[input_bit / 8] >> (input_bit % 8)) & 1;
                let h = aha_hash(&m, Domain::Hash, 32, r, constants, chi, rot);

                for output_bit in 0..n_output_bits {
                    let out_bit = (h[output_bit / 8] >> (output_bit % 8)) & 1;
                    if in_bit == out_bit {
                        equal_counts[output_bit] += 1;
                    }
                }
            }

            for output_bit in 0..n_output_bits {
                let p = equal_counts[output_bit] as f64 / samples_per_input_bit as f64;
                let bias = (p - 0.5).abs();
                matrix[input_bit][output_bit] = bias;
                max_bias = max_bias.max(bias);
                sum_bias += bias;
                cells += 1;
            }
        }

        round_global_max_bias.insert(r, max_bias);
        round_global_mean_bias.insert(r, sum_bias / cells as f64);
        rounds.insert(r, matrix);
    }

    LinearMatrixReport {
        n_input_bits,
        n_output_bits,
        samples_per_input_bit,
        rounds,
        round_global_max_bias,
        round_global_mean_bias,
    }
}


pub fn linear_correlation_probe(
    rounds_list: &[usize],
    samples: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> LinearReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    let input_bit = 0;
    let output_bit = 0;

    for &r in rounds_list {
        let mut count_equal = 0usize;

        for _ in 0..samples {
            let mut m = vec![0u8; msg_len];
            rng.fill(&mut m[..]);

            let in_bit = (m[input_bit / 8] >> (input_bit % 8)) & 1;

            let h = aha_hash(&m, Domain::Hash, 32, r, constants, chi, rot);
            let out_bit = (h[output_bit / 8] >> (output_bit % 8)) & 1;

            if in_bit == out_bit {
                count_equal += 1;
            }
        }

        let p = count_equal as f64 / samples as f64;
        let bias = (p - 0.5).abs();

        results.insert(r, LinearRoundStats {
            samples,
            input_bit,
            output_bit,
            bias,
        });
    }

    LinearReport { rounds: results }
}

pub fn structured_differential_search(
    rounds_list: &[usize],
    pair_count: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> StructuredDifferentialReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut out = HashMap::new();
    let patterns = ["single_bit", "adjacent_2", "adjacent_4", "same_byte_full", "same_lane_8"];
    let msg_bits = msg_len * 8;

    for &rounds in rounds_list {
        let mut per_pattern = HashMap::new();

        for pattern in patterns {
            let mut changed_bits = Vec::with_capacity(pair_count);
            let mut min_bits = u32::MAX;
            let mut max_bits = 0u32;
            let mut le32 = 0usize;
            let mut le48 = 0usize;
            let mut le64 = 0usize;

            for _ in 0..pair_count {
                let mut m = vec![0u8; msg_len];
                rng.fill(&mut m[..]);
                let mut m2 = m.clone();
                let base_pos = rng.gen_range(0..msg_bits);
                apply_structured_diff(&mut m2, pattern, base_pos, msg_bits);

                let h1 = aha_hash(&m, Domain::Hash, 32, rounds, constants, chi, rot);
                let h2 = aha_hash(&m2, Domain::Hash, 32, rounds, constants, chi, rot);
                let d = xor_bytes(&h1, &h2);
                let bits = popcount_bytes(&d);

                min_bits = min_bits.min(bits);
                max_bits = max_bits.max(bits);
                if bits <= 32 { le32 += 1; }
                if bits <= 48 { le48 += 1; }
                if bits <= 64 { le64 += 1; }
                changed_bits.push(bits as f64);
            }

            per_pattern.insert(pattern.to_string(), StructuredDifferentialRoundStats {
                pairs_per_pattern: pair_count,
                pattern: pattern.to_string(),
                min_changed_bits: min_bits,
                avg_changed_bits: changed_bits.iter().sum::<f64>() / changed_bits.len() as f64,
                max_changed_bits: max_bits,
                count_le_32: le32,
                count_le_48: le48,
                count_le_64: le64,
            });
        }

        out.insert(rounds, per_pattern);
    }

    StructuredDifferentialReport {
        variant: match chi {
            ChiVariant::Star => "spec_star_chi".to_string(),
            ChiVariant::Baseline => "baseline_chi".to_string(),
        },
        rounds: out,
    }
}


pub fn low_weight_differential_search(
    rounds_list: &[usize],
    pair_count: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> LowWeightReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut report = HashMap::new();
    for &rounds in rounds_list {
        let mut changed_bits = Vec::with_capacity(pair_count);
        let mut min_bits = u32::MAX;
        let mut max_bits = 0u32;
        let mut le32 = 0usize;
        let mut le48 = 0usize;
        let mut le64 = 0usize;
        for _ in 0..pair_count {
            let mut m = vec![0u8; msg_len];
            rng.fill(&mut m[..]);
            let mut m2 = m.clone();
            let pos = rng.gen_range(0..(msg_len * 8));
            m2[pos / 8] ^= 1u8 << (pos % 8);
            let h1 = aha_hash(&m, Domain::Hash, 32, rounds, constants, chi, rot);
            let h2 = aha_hash(&m2, Domain::Hash, 32, rounds, constants, chi, rot);
            let d = xor_bytes(&h1, &h2);
            let bits = popcount_bytes(&d);
            min_bits = min_bits.min(bits);
            max_bits = max_bits.max(bits);
            if bits <= 32 { le32 += 1; }
            if bits <= 48 { le48 += 1; }
            if bits <= 64 { le64 += 1; }
            changed_bits.push(bits as f64);
        }
        report.insert(rounds, LowWeightRoundStats {
            pairs: pair_count,
            min_changed_bits: min_bits,
            avg_changed_bits: changed_bits.iter().sum::<f64>() / changed_bits.len() as f64,
            max_changed_bits: max_bits,
            count_le_32: le32,
            count_le_48: le48,
            count_le_64: le64,
        });
    }
    LowWeightReport {
        variant: match chi {
            ChiVariant::Star => "spec_star_chi".to_string(),
            ChiVariant::Baseline => "baseline_chi".to_string(),
        },
        rounds: report,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CubeRoundStats {
    pub samples: usize,
    pub cube_bits: usize,
    pub output_parity_one_count: usize,
    pub output_parity_zero_count: usize,
    pub output_parity_unused_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CubeReport {
    pub rounds: HashMap<usize, CubeRoundStats>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct HigherOrderRoundStats {
    pub order: usize,
    pub pairs: usize,
    pub min_changed_bits: u32,
    pub avg_changed_bits: f64,
    pub max_changed_bits: u32,
    pub count_le_32: usize,
    pub count_le_48: usize,
    pub count_le_64: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HigherOrderReport {
    pub rounds: HashMap<usize, HigherOrderRoundStats>,
}

pub fn higher_order_differential_search(
    rounds_list: &[usize],
    pair_count: usize,
    msg_len: usize,
    order: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> HigherOrderReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut report = HashMap::new();

    for &rounds in rounds_list {
        let mut changed_bits = Vec::with_capacity(pair_count);
        let mut min_bits = u32::MAX;
        let mut max_bits = 0u32;
        let mut le32 = 0usize;
        let mut le48 = 0usize;
        let mut le64 = 0usize;

        for _ in 0..pair_count {
            let mut m = vec![0u8; msg_len];
            rng.fill(&mut m[..]);
            let mut m2 = m.clone();

            let mut positions = HashSet::new();
            while positions.len() < order {
                positions.insert(rng.gen_range(0..(msg_len * 8)));
            }
            for pos in positions {
                m2[pos / 8] ^= 1u8 << (pos % 8);
            }

            let h1 = aha_hash(&m, Domain::Hash, 32, rounds, constants, chi, rot);
            let h2 = aha_hash(&m2, Domain::Hash, 32, rounds, constants, chi, rot);
            let d = xor_bytes(&h1, &h2);
            let bits = popcount_bytes(&d);

            min_bits = min_bits.min(bits);
            max_bits = max_bits.max(bits);
            if bits <= 32 { le32 += 1; }
            if bits <= 48 { le48 += 1; }
            if bits <= 64 { le64 += 1; }
            changed_bits.push(bits as f64);
        }

        report.insert(rounds, HigherOrderRoundStats {
            order,
            pairs: pair_count,
            min_changed_bits: min_bits,
            avg_changed_bits: changed_bits.iter().sum::<f64>() / changed_bits.len() as f64,
            max_changed_bits: max_bits,
            count_le_32: le32,
            count_le_48: le48,
            count_le_64: le64,
        });
    }

    HigherOrderReport { rounds: report }
}


pub fn cube_probe(
    rounds_list: &[usize],
    samples: usize,
    msg_len: usize,
    cube_bits: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> CubeReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rounds_out = HashMap::new();
    let total_masks = 1usize << cube_bits;

    for &rounds in rounds_list {
        let mut parity_one = 0usize;
        let mut parity_zero = 0usize;
        let mut parity_unused = 0usize;

        for _ in 0..samples {
            let mut base = vec![0u8; msg_len];
            rng.fill(&mut base[..]);

            let mut positions = HashSet::new();
            while positions.len() < cube_bits {
                positions.insert(rng.gen_range(0..(msg_len * 8)));
            }
            let pos_vec: Vec<usize> = positions.into_iter().collect();

            let mut parity = vec![0u8; 256];
            for mask in 0..total_masks {
                let mut m = base.clone();
                for (j, pos) in pos_vec.iter().enumerate() {
                    if ((mask >> j) & 1) == 1 {
                        m[*pos / 8] ^= 1u8 << (*pos % 8);
                    }
                }
                let h = aha_hash(&m, Domain::Hash, 32, rounds, constants, chi, rot);
                for i in 0..256 {
                    parity[i] ^= (h[i / 8] >> (i % 8)) & 1;
                }
            }

            for bit in parity {
                if bit == 0 {
                    parity_zero += 1;
                } else {
                    parity_one += 1;
                }
            }
        }

        rounds_out.insert(rounds, CubeRoundStats {
            samples,
            cube_bits,
            output_parity_one_count: parity_one,
            output_parity_zero_count: parity_zero,
            output_parity_unused_count: parity_unused,
        });
    }

    CubeReport { rounds: rounds_out }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LaneActivityRoundStats {
    pub rounds: usize,
    pub samples: usize,
    pub active_output_bits_avg: f64,
    pub active_output_bits_min: u32,
    pub active_output_bits_max: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaneActivityReport {
    pub rounds: HashMap<usize, LaneActivityRoundStats>,
}

pub fn lane_activity_probe(
    rounds_list: &[usize],
    samples: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> LaneActivityReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut report = HashMap::new();

    for &r in rounds_list {
        let mut counts = Vec::with_capacity(samples);
        let mut min_bits = u32::MAX;
        let mut max_bits = 0u32;

        for _ in 0..samples {
            let mut m = vec![0u8; msg_len];
            rng.fill(&mut m[..]);
            let mut m2 = m.clone();

            let pos = rng.gen_range(0..(msg_len * 8));
            m2[pos / 8] ^= 1u8 << (pos % 8);

            let h1 = aha_hash(&m, Domain::Hash, 32, r, constants, chi, rot);
            let h2 = aha_hash(&m2, Domain::Hash, 32, r, constants, chi, rot);

            let d = xor_bytes(&h1, &h2);
            let bits = popcount_bytes(&d);

            min_bits = min_bits.min(bits);
            max_bits = max_bits.max(bits);
            counts.push(bits as f64);
        }

        report.insert(r, LaneActivityRoundStats {
            rounds: r,
            samples,
            active_output_bits_avg: counts.iter().sum::<f64>() / counts.len() as f64,
            active_output_bits_min: min_bits,
            active_output_bits_max: max_bits,
        });
    }

    LaneActivityReport { rounds: report }
}


pub fn stronger_reduced_round_search(
    rounds_list: &[usize],
    pair_count: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> ReducedRoundReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut report = HashMap::new();
    for &rounds in rounds_list {
        let mut diffs: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut changed = Vec::with_capacity(pair_count);
        let mut zero = 0usize;
        for _ in 0..pair_count {
            let mut m = vec![0u8; msg_len];
            rng.fill(&mut m[..]);
            let mut m2 = m.clone();
            let pos = rng.gen_range(0..(msg_len * 8));
            m2[pos / 8] ^= 1u8 << (pos % 8);
            let h1 = aha_hash(&m, Domain::Hash, 32, rounds, constants, chi, rot);
            let h2 = aha_hash(&m2, Domain::Hash, 32, rounds, constants, chi, rot);
            let d = xor_bytes(&h1, &h2);
            if d.iter().all(|b| *b == 0) {
                zero += 1;
            }
            *diffs.entry(d.clone()).or_insert(0) += 1;
            changed.push(popcount_bytes(&d) as f64 / 256.0);
        }
        let mut counts: Vec<usize> = diffs.values().copied().collect();
        counts.sort_unstable_by(|a, b| b.cmp(a));
        let stats = RoundStats {
            pairs: pair_count,
            unique_output_differences: diffs.len(),
            max_repeated_output_difference_count: counts.first().copied().unwrap_or(0),
            top5_repeat_counts: counts.into_iter().take(5).collect(),
            zero_difference_count: zero,
            avg_changed_fraction: changed.iter().sum::<f64>() / changed.len() as f64,
            min_changed_fraction: changed.iter().copied().fold(f64::INFINITY, f64::min),
            max_changed_fraction: changed.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        };
        report.insert(rounds, stats);
    }
    ReducedRoundReport {
        variant: match chi {
            ChiVariant::Star => "spec_star_chi".to_string(),
            ChiVariant::Baseline => "baseline_chi".to_string(),
        },
        rounds: report,
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AvalancheMatrixReport {
    pub n_input_bits: usize,
    pub n_output_bits: usize,
    pub pairs_per_input_bit: usize,
    pub matrix: Vec<Vec<f64>>,
    pub global_min_prob: f64,
    pub global_max_prob: f64,
    pub global_mean_abs_dev: f64,
    pub global_max_abs_dev: f64,
}

pub fn avalanche_matrix_stats(
    msg_len: usize,
    n_input_bits: usize,
    n_msgs_per_input: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> AvalancheMatrixReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let n_output_bits = 256usize;
    let mut counts = vec![vec![0usize; n_output_bits]; n_input_bits];

    for input_bit in 0..n_input_bits {
        for _ in 0..n_msgs_per_input {
            let mut msg = vec![0u8; msg_len];
            rng.fill(&mut msg[..]);
            let base = aha_hash(&msg, Domain::Hash, 32, ROUNDS, constants, chi, rot);
            let mut m2 = msg.clone();
            m2[input_bit / 8] ^= 1u8 << (input_bit % 8);
            let h2 = aha_hash(&m2, Domain::Hash, 32, ROUNDS, constants, chi, rot);
            let diff = xor_bytes(&base, &h2);
            for (j, byte) in diff.iter().enumerate() {
                for k in 0..8 {
                    if (byte >> k) & 1 == 1 {
                        counts[input_bit][j * 8 + k] += 1;
                    }
                }
            }
        }
    }

    let matrix: Vec<Vec<f64>> = counts.iter()
        .map(|row| row.iter().map(|&c| c as f64 / n_msgs_per_input as f64).collect())
        .collect();

    let mut global_min_prob = f64::INFINITY;
    let mut global_max_prob = f64::NEG_INFINITY;
    let mut global_abs_dev_sum = 0.0f64;
    let mut global_max_abs_dev = 0.0f64;
    let mut total_cells = 0usize;

    for row in &matrix {
        for &p in row {
            global_min_prob = global_min_prob.min(p);
            global_max_prob = global_max_prob.max(p);
            let d = (p - 0.5).abs();
            global_abs_dev_sum += d;
            global_max_abs_dev = global_max_abs_dev.max(d);
            total_cells += 1;
        }
    }

    AvalancheMatrixReport {
        n_input_bits,
        n_output_bits,
        pairs_per_input_bit: n_msgs_per_input,
        matrix,
        global_min_prob,
        global_max_prob,
        global_mean_abs_dev: global_abs_dev_sum / total_cells as f64,
        global_max_abs_dev,
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AvalancheReport {
    pub pairs: usize,
    pub avg_changed_bits: f64,
    pub avg_changed_fraction: f64,
    pub output_flip_mean_prob: f64,
    pub output_flip_mean_abs_dev: f64,
    pub output_flip_max_abs_dev: f64,
    pub output_flip_min_prob: f64,
    pub output_flip_max_prob: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvalancheRoundReport {
    pub rounds: HashMap<usize, AvalancheReport>,
}



pub fn avalanche_round_stats(
    rounds_list: &[usize],
    msg_len: usize,
    n_msgs: usize,
    flips_per_msg: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> AvalancheRoundReport {
    let mut out = HashMap::new();

    for &rounds in rounds_list {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut total_pairs = 0usize;
        let mut changed = Vec::with_capacity(n_msgs * flips_per_msg);
        let mut out_flip_counts = vec![0usize; 256];

        for _ in 0..n_msgs {
            let mut msg = vec![0u8; msg_len];
            rng.fill(&mut msg[..]);
            let base = aha_hash(&msg, Domain::Hash, 32, rounds, constants, chi, rot);
            let mut chosen = HashSet::new();
            while chosen.len() < flips_per_msg {
                chosen.insert(rng.gen_range(0..msg_len * 8));
            }
            for pos in chosen {
                let mut m2 = msg.clone();
                m2[pos / 8] ^= 1u8 << (pos % 8);
                let h2 = aha_hash(&m2, Domain::Hash, 32, rounds, constants, chi, rot);
                let diff = xor_bytes(&base, &h2);
                changed.push(popcount_bytes(&diff) as usize);
                for (j, byte) in diff.iter().enumerate() {
                    for k in 0..8 {
                        if (byte >> k) & 1 == 1 {
                            out_flip_counts[j * 8 + k] += 1;
                        }
                    }
                }
                total_pairs += 1;
            }
        }

        let probs: Vec<f64> = out_flip_counts.iter().map(|&c| c as f64 / total_pairs as f64).collect();
        out.insert(rounds, AvalancheReport {
            pairs: total_pairs,
            avg_changed_bits: changed.iter().sum::<usize>() as f64 / changed.len() as f64,
            avg_changed_fraction: changed.iter().sum::<usize>() as f64 / (changed.len() * 256) as f64,
            output_flip_mean_prob: probs.iter().sum::<f64>() / 256.0,
            output_flip_mean_abs_dev: probs.iter().map(|p| (p - 0.5).abs()).sum::<f64>() / 256.0,
            output_flip_max_abs_dev: probs.iter().map(|p| (p - 0.5).abs()).fold(0.0_f64, f64::max),
            output_flip_min_prob: probs.iter().copied().fold(f64::INFINITY, f64::min),
            output_flip_max_prob: probs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        });
    }

    AvalancheRoundReport { rounds: out }
}


pub fn avalanche_stats(
    msg_len: usize,
    n_msgs: usize,
    flips_per_msg: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> AvalancheReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut total_pairs = 0usize;
    let mut changed = Vec::with_capacity(n_msgs * flips_per_msg);
    let mut out_flip_counts = vec![0usize; 256];
    for _ in 0..n_msgs {
        let mut msg = vec![0u8; msg_len];
        rng.fill(&mut msg[..]);
        let base = aha_hash(&msg, Domain::Hash, 32, ROUNDS, constants, chi, rot);
        let mut chosen = HashSet::new();
        while chosen.len() < flips_per_msg {
            chosen.insert(rng.gen_range(0..msg_len * 8));
        }
        for pos in chosen {
            let mut m2 = msg.clone();
            m2[pos / 8] ^= 1u8 << (pos % 8);
            let h2 = aha_hash(&m2, Domain::Hash, 32, ROUNDS, constants, chi, rot);
            let diff = xor_bytes(&base, &h2);
            changed.push(popcount_bytes(&diff) as usize);
            for (j, byte) in diff.iter().enumerate() {
                for k in 0..8 {
                    if (byte >> k) & 1 == 1 {
                        out_flip_counts[j * 8 + k] += 1;
                    }
                }
            }
            total_pairs += 1;
        }
    }
    let probs: Vec<f64> = out_flip_counts.iter().map(|&c| c as f64 / total_pairs as f64).collect();
    AvalancheReport {
        pairs: total_pairs,
        avg_changed_bits: changed.iter().sum::<usize>() as f64 / changed.len() as f64,
        avg_changed_fraction: changed.iter().sum::<usize>() as f64 / (changed.len() * 256) as f64,
        output_flip_mean_prob: probs.iter().sum::<f64>() / 256.0,
        output_flip_mean_abs_dev: probs.iter().map(|p| (p - 0.5).abs()).sum::<f64>() / 256.0,
        output_flip_max_abs_dev: probs.iter().map(|p| (p - 0.5).abs()).fold(0.0_f64, f64::max),
        output_flip_min_prob: probs.iter().copied().fold(f64::INFINITY, f64::min),
        output_flip_max_prob: probs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnfRoundSummary {
    pub round: usize,
    pub max_degree: usize,
    pub min_degree: usize,
    pub avg_degree: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnfExperimentReport {
    pub lane_width: usize,
    pub state_bits: usize,
    pub tracked_outputs: usize,
    pub summaries: Vec<AnfRoundSummary>,
}

fn mobius_anf_degree(mut table: Vec<u8>, vars: usize) -> usize {
    let n = 1usize << vars;
    for i in 0..vars {
        for mask in 0..n {
            if (mask & (1 << i)) != 0 {
                table[mask] ^= table[mask ^ (1 << i)];
            }
        }
    }
    let mut max_deg = 0usize;
    for (mask, &coef) in table.iter().enumerate() {
        if coef & 1 == 1 {
            max_deg = max_deg.max(mask.count_ones() as usize);
        }
    }
    max_deg
}

fn state_from_small_bits(bits: &[u8], lane_width: usize) -> State {
    let mut s = blank_state();
    let lanes_used = bits.len() / lane_width;
    let mut idx = 0usize;
    for lane in 0..lanes_used {
        let x = lane % 5;
        let y = lane / 5;
        let mut lane_bits = 0u64;
        for b in 0..lane_width {
            lane_bits |= (bits[idx] as u64) << b;
            idx += 1;
        }
        s[x][y] = lane_bits;
    }
    s
}

pub fn exact_small_width_anf_experiment(
    lane_width: usize,
    rounds: usize,
    tracked_outputs: usize,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> AnfExperimentReport {
    let lanes_used = 4usize;
    let vars = lanes_used * lane_width;
    assert!(vars <= 16, "exact truth-table ANF explodes past 16 vars; choose smaller width or fewer lanes");
    let n = 1usize << vars;
    let mut summaries = Vec::new();
    for r in 1..=rounds {
        let mut degrees = Vec::new();
        for out_idx in 0..tracked_outputs {
            let lane_idx = out_idx / lane_width;
            let bit_idx = out_idx % lane_width;
            let out_x = lane_idx % 5;
            let out_y = lane_idx / 5;
            let mut table = vec![0u8; n];
            for mask in 0..n {
                let mut bits = vec![0u8; vars];
                for i in 0..vars {
                    bits[i] = ((mask >> i) & 1) as u8;
                }
                let s0 = state_from_small_bits(&bits, lane_width);
                let sr = permute(s0, r, constants, chi, rot);
                table[mask] = ((sr[out_x][out_y] >> bit_idx) & 1) as u8;
            }
            degrees.push(mobius_anf_degree(table, vars));
        }
        let min_degree = *degrees.iter().min().unwrap();
        let max_degree = *degrees.iter().max().unwrap();
        let avg_degree = degrees.iter().sum::<usize>() as f64 / degrees.len() as f64;
        summaries.push(AnfRoundSummary { round: r, max_degree, min_degree, avg_degree });
    }
    AnfExperimentReport {
        lane_width,
        state_bits: vars,
        tracked_outputs,
        summaries,
    }
}

pub fn shifted_rot() -> [[u32; 5]; 5] {
    let mut out = [[0u32; 5]; 5];
    for x in 0..5 {
        for y in 0..5 {
            out[x][y] = (ROT[x][y] + 1) % 64;
        }
    }
    out
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RotationReport {
    pub tested_rotations: usize,
    pub rounds: HashMap<usize, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FixedPointReport {
    pub samples: usize,
    pub rounds: HashMap<usize, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CycleReport {
    pub samples: usize,
    pub rounds: HashMap<usize, usize>, // count of cycles found
}

pub fn two_cycle_search(
    rounds_list: &[usize],
    samples: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> CycleReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    for &r in rounds_list {
        let mut found = 0usize;

        for _ in 0..samples {
            let mut s = blank_state();
            for x in 0..5 {
                for y in 0..5 {
                    s[x][y] = rng.gen::<u64>();
                }
            }

            let s1 = permute(s, r, constants, chi, rot);
            let s2 = permute(s1, r, constants, chi, rot);

            if s2 == s && s1 != s {
                found += 1;
            }
        }

        results.insert(r, found);
    }

    CycleReport { samples, rounds: results }
}

pub fn three_cycle_search(
    rounds_list: &[usize],
    samples: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> CycleReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    for &r in rounds_list {
        let mut found = 0usize;

        for _ in 0..samples {
            let mut s = blank_state();
            for x in 0..5 {
                for y in 0..5 {
                    s[x][y] = rng.gen::<u64>();
                }
            }

            let s1 = permute(s, r, constants, chi, rot);
            let s2 = permute(s1, r, constants, chi, rot);
            let s3 = permute(s2, r, constants, chi, rot);

            if s3 == s && s1 != s && s2 != s {
                found += 1;
            }
        }

        results.insert(r, found);
    }

    CycleReport { samples, rounds: results }
}

pub fn four_cycle_search(
    rounds_list: &[usize],
    samples: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> CycleReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    for &r in rounds_list {
        let mut found = 0usize;

        for _ in 0..samples {
            let mut s = blank_state();
            for x in 0..5 {
                for y in 0..5 {
                    s[x][y] = rng.gen::<u64>();
                }
            }

            let s1 = permute(s, r, constants, chi, rot);
            let s2 = permute(s1, r, constants, chi, rot);
            let s3 = permute(s2, r, constants, chi, rot);
            let s4 = permute(s3, r, constants, chi, rot);

            if s4 == s && s1 != s && s2 != s && s3 != s {
                found += 1;
            }
        }

        results.insert(r, found);
    }

    CycleReport { samples, rounds: results }
}

pub fn fixed_point_search(
    rounds_list: &[usize],
    samples: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> FixedPointReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    for &r in rounds_list {
        let mut found = 0usize;

        for _ in 0..samples {
            let mut s = blank_state();

            for x in 0..5 {
                for y in 0..5 {
                    s[x][y] = rng.gen::<u64>();
                }
            }

            let sr = permute(s, r, constants, chi, rot);

            if sr == s {
                found += 1;
            }
        }

        results.insert(r, found);
    }

    FixedPointReport {
        samples,
        rounds: results,
    }
}


pub fn rotation_test(
    rounds_list: &[usize],
    samples: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> RotationReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    for &r in rounds_list {
        let mut survives = 0usize;

        for _ in 0..samples {
            let mut msg = vec![0u8; msg_len];
            rng.fill(&mut msg[..]);
            let h = aha_hash(&msg, Domain::Hash, 32, r, constants, chi, rot);

            for rot_bits in 1..64 {
                if rot_bits % 8 == 0 { continue; }
                let mut rotated = msg.clone();
                for byte in &mut rotated {
                    *byte = byte.rotate_left(rot_bits as u32);
                }

                let h_rot = aha_hash(&rotated, Domain::Hash, 32, r, constants, chi, rot);

                let mut match_rot = true;
                for (a, b) in h.iter().zip(h_rot.iter()) {
                    if b.rotate_right(rot_bits as u32) != *a {
                        match_rot = false;
                        break;
                    }
                }

                if match_rot {
                    survives += 1;
                }
            }
        }

        results.insert(r, survives);
    }

    RotationReport {
        tested_rotations: 63,
        rounds: results,
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SatLikeRoundStats {
    pub rounds: usize,
    pub samples: usize,
    pub unique_output_differences: usize,
    pub max_repeated_output_difference_count: usize,
    pub avg_changed_fraction: f64,
    pub min_changed_fraction: f64,
    pub max_changed_fraction: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SatLikeReport {
    pub samples: usize,
    pub rounds: HashMap<usize, SatLikeRoundStats>,
}

pub fn sat_like_reduced_round_structure(
    rounds_list: &[usize],
    samples: usize,
    msg_len: usize,
    seed: u64,
    constants: &Constants,
    chi: ChiVariant,
    rot: &[[u32; 5]; 5],
) -> SatLikeReport {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut results = HashMap::new();

    for &r in rounds_list {
        let mut diffs: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut changed = Vec::with_capacity(samples);

        for _ in 0..samples {
            let mut m = vec![0u8; msg_len];
            rng.fill(&mut m[..]);
            let mut m2 = m.clone();

            let mut positions = HashSet::new();
            while positions.len() < 2 {
                positions.insert(rng.gen_range(0..(msg_len * 8)));
            }
            for pos in positions {
                m2[pos / 8] ^= 1u8 << (pos % 8);
            }

            let h1 = aha_hash(&m, Domain::Hash, 32, r, constants, chi, rot);
            let h2 = aha_hash(&m2, Domain::Hash, 32, r, constants, chi, rot);
            let d = xor_bytes(&h1, &h2);

            *diffs.entry(d.clone()).or_insert(0) += 1;
            changed.push(popcount_bytes(&d) as f64 / 256.0);
        }

        let mut counts: Vec<usize> = diffs.values().copied().collect();
        counts.sort_unstable_by(|a, b| b.cmp(a));

        let min_changed_fraction = changed.iter().copied().fold(f64::INFINITY, f64::min);
        let max_changed_fraction = changed.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        results.insert(r, SatLikeRoundStats {
            rounds: r,
            samples,
            unique_output_differences: diffs.len(),
            max_repeated_output_difference_count: counts.first().copied().unwrap_or(0),
            avg_changed_fraction: changed.iter().sum::<f64>() / changed.len() as f64,
            min_changed_fraction,
            max_changed_fraction,
        });
    }

    SatLikeReport { samples, rounds: results }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_python_summary() {
        let c = derive_constants();
        assert_eq!(c.k0[0], 0x1574_243b_711d_5566);
        assert_eq!(c.k1[0], 0x5295_4354_2562_3498);
        assert_eq!(c.k2[0], 0x981e_63bd_9227_548f);
    }

    #[test]
    fn hash_vectors_match_python() {
        let c = derive_constants();
        assert_eq!(hex_of(&aha_hash(b"", Domain::Hash, 32, ROUNDS, &c, ChiVariant::Star, &ROT)), "e8bf66fb70ec3787817c0cb717952140569a853f94dee36a21268632b9a59ed0");
        assert_eq!(hex_of(&aha_hash(b"abc", Domain::Hash, 32, ROUNDS, &c, ChiVariant::Star, &ROT)), "50f4f48736c87a32bb20c618fda7de0ec0260edd57f340e92d8daa45d54a4a1f");
        assert_eq!(hex_of(&aha_hash(&vec![0u8;128], Domain::Hash, 32, ROUNDS, &c, ChiVariant::Star, &ROT)), "22598b6298b7125bdacf7486508d3efc34e93334f93b889b736e2614cd3479fe");
        let mut m = vec![0u8;128]; m[127] = 1;
        assert_eq!(hex_of(&aha_hash(&m, Domain::Hash, 32, ROUNDS, &c, ChiVariant::Star, &ROT)), "2eb15de636e671274ffe8891dae56353712dc4fbffca2876041d2d63219ec5dc");
    }

    #[test]
    fn xof_vectors_match_python() {
        let c = derive_constants();
        assert_eq!(hex_of(&aha_hash(b"", Domain::Xof, 64, ROUNDS, &c, ChiVariant::Star, &ROT)), "01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76");
        assert_eq!(hex_of(&aha_hash(b"abc", Domain::Xof, 64, ROUNDS, &c, ChiVariant::Star, &ROT)), "87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f5410");
    }
}
