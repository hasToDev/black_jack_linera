// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Mutex, OnceLock};
use rand::{rngs::StdRng, Rng, SeedableRng};

static RNG: OnceLock<Mutex<StdRng>> = OnceLock::new();

pub fn custom_getrandom(buf: &mut [u8], seed: [u8; 32]) -> Result<(), getrandom::Error> {
    RNG.get_or_init(|| Mutex::new(StdRng::from_seed(seed)))
        .lock()
        .expect("failed to get RNG lock")
        .fill(buf);
    Ok(())
}

pub fn generate_range(seed: [u8; 32], max_value: u8) -> u8 {
    RNG.get_or_init(|| Mutex::new(StdRng::from_seed(seed)))
        .lock()
        .expect("failed to get RNG lock")
        .gen_range(1..max_value)
}

pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

pub fn random_index(timestamp: String, length: u8, id: String, seed: String) -> u8 {
    // produce seed array using system time
    let concatenated_timestamp = format!("{}{}{}{}{}", id, seed, timestamp, timestamp, timestamp);
    let timestamp_str = truncate(concatenated_timestamp.as_str(), 32);
    let mut seed_array = [0u8; 32];
    seed_array = <[u8; 32]>::try_from(timestamp_str.as_bytes()).unwrap();

    // get random index using provided seed
    generate_range(seed_array, length)
}

