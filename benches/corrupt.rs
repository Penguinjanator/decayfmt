//! Manual timing benchmark for the corruption loop.
//!
//! Runs `corrupt::corrupt` over a deterministic pattern-filled payload for the
//! four scenarios {text, image} x {x = 1, x = 10}, adapting the repetition count
//! until each scenario has run for at least ~1.5 s, then reports MiB/s. The
//! payload fill uses a cheap xorshift generator so the timing measures the
//! corruption pass itself, not entropy seeding or allocation.
//!
//! Usage: `cargo bench --bench corrupt` (release profile by default).
//! Payload size defaults to 64 MiB; override with `DECAY_BENCH_MB=256 cargo bench --bench corrupt`.

use decayfmt::corrupt::corrupt;
use decayfmt::format::FileType;
use std::time::Instant;

const DEFAULT_PAYLOAD_MB: usize = 64;
const MIN_SCENARIO_SECS: f64 = 1.5;

/// Fills `bytes` with a cheap deterministic pattern (xorshift64). Deterministic
/// so runs are comparable; not cryptographically strong, which is fine because
/// it never feeds the corruption RNG — `corrupt` seeds its own from OS entropy.
fn fill_pattern(bytes: &mut [u8]) {
    let mut state = 0x243F_6A88_85A3_08D3u64;
    for chunk in bytes.chunks_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let words = state.to_le_bytes();
        let n = chunk.len();
        chunk.copy_from_slice(&words[..n]);
    }
}

/// Times one scenario, adapting the repetition count until `MIN_SCENARIO_SECS`
/// has elapsed, and prints throughput.
fn bench_scenario(payload: &mut [u8], file_type: FileType, x: f64, payload_mb: f64) {
    // Warmup: one untimed pass so caches and code paths are hot.
    corrupt(payload, file_type, x);

    let mut reps = 1u64;
    let (elapsed, count) = loop {
        let start = Instant::now();
        for _ in 0..reps {
            corrupt(payload, file_type, x);
        }
        let elapsed = start.elapsed();
        if elapsed.as_secs_f64() >= MIN_SCENARIO_SECS {
            break (elapsed, reps);
        }
        reps *= 2;
    };

    let gib_per_sec = payload_mb * count as f64 / elapsed.as_secs_f64() / 1024.0;
    let mib_per_sec = payload_mb * count as f64 / elapsed.as_secs_f64();
    println!(
        "{:>5} x={:<3}: {:>8.1} MiB/s  ({:>6} reps in {:>6.2} s, payload {:.0} MiB)",
        file_type.label(),
        x,
        mib_per_sec,
        count,
        elapsed.as_secs_f64(),
        payload_mb,
    );
    println!("        x={:<3}: {:>8.3} GiB/s", x, gib_per_sec);
}

fn main() {
    let payload_mb = std::env::var("DECAY_BENCH_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PAYLOAD_MB);
    let payload_len = payload_mb * 1024 * 1024;
    let payload_mb_f = payload_mb as f64;

    let mut payload = vec![0u8; payload_len];
    fill_pattern(&mut payload);
    println!(
        "corruption loop benchmark — payload {} MiB ({} bytes), scenarios until >= {:.1} s each",
        payload_mb, payload_len, MIN_SCENARIO_SECS
    );
    println!("cores reported by rayon: {}", rayon::current_num_threads());

    for (file_type, x) in [
        (FileType::Text, 1.0),
        (FileType::Text, 10.0),
        (FileType::Image, 1.0),
        (FileType::Image, 10.0),
    ] {
        bench_scenario(&mut payload, file_type, x, payload_mb_f);
    }
}
