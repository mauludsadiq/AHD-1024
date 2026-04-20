use ahd_1024::*;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

fn usage() {
    eprintln!(
        "Usage:\n  cargo run --release -- vectors\n  cargo run --release -- cross-check\n  cargo run --release -- reduced-search [pairs] [msg_len] [seed]\n  cargo run --release -- avalanche [n_msgs] [flips_per_msg] [msg_len] [seed]\n  cargo run --release -- anf-small [lane_width] [rounds] [tracked_outputs]\n"
    );
}

fn results_dir() -> PathBuf {
    let p = PathBuf::from("results");
    let _ = fs::create_dir_all(&p);
    p
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        return;
    }
    let constants = derive_constants();
    match args[1].as_str() {
        "trace-diff" => {
            let rounds = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(3);
            let bit = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let mut delta = blank_state();
            delta[(bit / 64) % 5][(bit / 320) % 5] = 1u64 << (bit % 64);
            let report = trace_difference(delta, rounds, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("trace_diff_r{}_bit{}.json", rounds, bit));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "trace-diff-pair" => {
            let rounds = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(3);
            let bit1 = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let bit2 = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            let mut delta = blank_state();
            delta[(bit1 / 64) % 5][(bit1 / 320) % 5] ^= 1u64 << (bit1 % 64);
            delta[(bit2 / 64) % 5][(bit2 / 320) % 5] ^= 1u64 << (bit2 % 64);
            let report = trace_difference(delta, rounds, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("trace_diff_pair_r{}_b{}_{}.json", rounds, bit1, bit2));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "vectors" => {
            let out = json!({
                "constants": {
                    "K0_0": format!("0x{:016x}", constants.k0[0]),
                    "K1_0": format!("0x{:016x}", constants.k1[0]),
                    "K2_0": format!("0x{:016x}", constants.k2[0]),
                },
                "HASH": {
                    "empty": hex_of(&aha_hash(b"", Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "a": hex_of(&aha_hash(b"a", Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "abc": hex_of(&aha_hash(b"abc", Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "alphabet52": hex_of(&aha_hash(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ", Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "zero126": hex_of(&aha_hash(&vec![0u8; 126], Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "zero128": hex_of(&aha_hash(&vec![0u8; 128], Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "ff128": hex_of(&aha_hash(&vec![0xffu8; 128], Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                },
                "XOF64": {
                    "empty": hex_of(&aha_hash(b"", Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "a": hex_of(&aha_hash(b"a", Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "abc": hex_of(&aha_hash(b"abc", Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "alphabet52": hex_of(&aha_hash(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ", Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "zero126": hex_of(&aha_hash(&vec![0u8; 126], Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "zero128": hex_of(&aha_hash(&vec![0u8; 128], Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                    "ff128": hex_of(&aha_hash(&vec![0xffu8; 128], Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                }
            });
            let path = results_dir().join("vectors.json");
            fs::write(&path, serde_json::to_vec_pretty(&out).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        "cross-check" => {
            let report = json!({
                "hash_empty": hex_of(&aha_hash(b"", Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                "hash_abc": hex_of(&aha_hash(b"abc", Domain::Hash, 32, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                "xof_empty_64": hex_of(&aha_hash(b"", Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                "xof_abc_64": hex_of(&aha_hash(b"abc", Domain::Xof, 64, ROUNDS, &constants, ChiVariant::Star, &ROT)),
                "expected": {
                    "hash_empty": "e8bf66fb70ec3787817c0cb717952140569a853f94dee36a21268632b9a59ed0",
                    "hash_abc": "50f4f48736c87a32bb20c618fda7de0ec0260edd57f340e92d8daa45d54a4a1f",
                    "xof_empty_64": "01e22fe9b943da60f3e76b18355c459d3374e02bbf6db61929ad7991edc0f08462ab96efcbfc0e83af22d1f17227f4c22948188749ad465f84cd037048ed8b76",
                    "xof_abc_64": "87b3ebdd896a889f6bc6fc52482470205bc63c68c5ab101c500c4aa4d044e891043b1e6bc9a00f313585beba4de91cdf86f2d351792e8685ebf8b427097f5410"
                }
            });
            let path = results_dir().join("cross_check.json");
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "reduced-search" => {
            let pairs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20_000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = stronger_reduced_round_search(&[1, 2, 3, 4, 5, 6], pairs, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("reduced_search_pairs{}_msg{}_seed{}.json", pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "reduced-search-shifted" => {
            let pairs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20_000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let rot = shifted_rot();
            let report = stronger_reduced_round_search(&[1, 2, 3, 4, 5, 6], pairs, msg_len, seed, &constants, ChiVariant::Star, &rot);
            let path = results_dir().join(format!("reduced_search_shifted_pairs{}_msg{}_seed{}.json", pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "avalanche" => {
            let n_msgs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(512);
            let flips = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(32);
            let msg_len = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(5).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let report = avalanche_stats(msg_len, n_msgs, flips, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("avalanche_msgs{}_flips{}_msg{}_seed{}.json", n_msgs, flips, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "avalanche-24" => {
            let n_msgs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1024);
            let flips = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(32);
            let msg_len = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(5).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let rounds: Vec<usize> = (1..=24).collect();
            let report = avalanche_round_stats(&rounds, msg_len, n_msgs, flips, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("avalanche_24_msgs{}_flips{}_msg{}_seed{}.json", n_msgs, flips, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "avalanche-matrix" => {
            let n_input_bits = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(64);
            let n_msgs_per_input = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1024);
            let msg_len = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(5).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let report = avalanche_matrix_stats(msg_len, n_input_bits, n_msgs_per_input, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("avalanche_matrix_in{}_msgs{}_msg{}_seed{}.json", n_input_bits, n_msgs_per_input, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "anf-small" => {
            let lane_width = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            let rounds = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(6);
            let tracked_outputs = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(8);
            let report = exact_small_width_anf_experiment(lane_width, rounds, tracked_outputs, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("anf_small_w{}_r{}_o{}.json", lane_width, rounds, tracked_outputs));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "rotation-test" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = rotation_test(&[1, 2, 3, 4, 5, 6], samples, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("rotation_test_samples{}_msg{}_seed{}.json", samples, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "fixed-point" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1_000_000);
            let seed = args.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = fixed_point_search(&[1,2,3,4,5,6], samples, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("fixed_points_samples{}_seed{}.json", samples, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "low-weight" => {
            let pairs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(200_000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = low_weight_differential_search(&[1,2,3,4,5,6], pairs, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("low_weight_pairs{}_msg{}_seed{}.json", pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "low-weight-baseline" => {
            let pairs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(200_000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = low_weight_differential_search(&[1,2,3,4,5,6], pairs, msg_len, seed, &constants, ChiVariant::Baseline, &ROT);
            let path = results_dir().join(format!("low_weight_baseline_pairs{}_msg{}_seed{}.json", pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "higher-order" => {
            let order = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(2);
            let pairs = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
            let msg_len = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(5).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let report = higher_order_differential_search(&[1,2,3,4,5,6], pairs, msg_len, order, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("higher_order_o{}_pairs{}_msg{}_seed{}.json", order, pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "cube" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(256);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let cube_bits = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(4);
            let seed = args.get(5).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = cube_probe(&[1,2,3,4,5,6], samples, msg_len, cube_bits, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("cube_samples{}_msg{}_k{}_seed{}.json", samples, msg_len, cube_bits, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "two-cycle" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(200000);
            let seed = args.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = two_cycle_search(&[1,2,3,4,5,6], samples, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("two_cycle_samples{}_seed{}.json", samples, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "three-cycle" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(200000);
            let seed = args.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = three_cycle_search(&[1,2,3,4,5,6], samples, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("three_cycle_samples{}_seed{}.json", samples, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "four-cycle" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(200000);
            let seed = args.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = four_cycle_search(&[1,2,3,4,5,6], samples, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("four_cycle_samples{}_seed{}.json", samples, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "sat-like" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let report = sat_like_reduced_round_structure(&[1,2,3,4,5,6], samples, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("sat_like_samples{}_msg{}_seed{}.json", samples, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "lane-activity" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let report = lane_activity_probe(&[1,2,3,4,5,6], samples, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("lane_activity_samples{}_msg{}_seed{}.json", samples, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "structured-diff" => {
            let pairs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = structured_differential_search(&[1,2,3,4], pairs, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("structured_diff_pairs{}_msg{}_seed{}.json", pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "structured-diff-baseline" => {
            let pairs = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = structured_differential_search(&[1,2,3,4], pairs, msg_len, seed, &constants, ChiVariant::Baseline, &ROT);
            let path = results_dir().join(format!("structured_diff_baseline_pairs{}_msg{}_seed{}.json", pairs, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "linear-probe" => {
            let samples = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(200000);
            let msg_len = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(7);
            let report = linear_correlation_probe(&[1,2,3,4,5,6], samples, msg_len, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("linear_probe_samples{}_msg{}_seed{}.json", samples, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "linear-matrix" => {
            let n_input_bits = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(64);
            let n_output_bits = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(256);
            let samples_per_input = args.get(4).and_then(|s| s.parse::<usize>().ok()).unwrap_or(4096);
            let msg_len = args.get(5).and_then(|s| s.parse::<usize>().ok()).unwrap_or(96);
            let seed = args.get(6).and_then(|s| s.parse::<u64>().ok()).unwrap_or(1234);
            let report = linear_correlation_matrix(&[1,2,3,4,5,6], samples_per_input, msg_len, n_input_bits, n_output_bits, seed, &constants, ChiVariant::Star, &ROT);
            let path = results_dir().join(format!("linear_matrix_in{}_out{}_samples{}_msg{}_seed{}.json", n_input_bits, n_output_bits, samples_per_input, msg_len, seed));
            fs::write(&path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            println!("wrote {}", path.display());
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        _ => usage(),
    }
}
