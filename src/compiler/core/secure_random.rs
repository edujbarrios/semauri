// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

// CSPRNG obtained directly from the operating system. No time-based seeds,
// PRNG fallback, predictable simulation or external Rust dependencies.
#[cfg(unix)]
struct OsSecureRandom(std::fs::File);
#[cfg(windows)]
struct OsSecureRandom;

#[cfg(unix)]
impl OsSecureRandom {
    fn new() -> std::io::Result<Self> {
        std::fs::File::open("/dev/urandom").map(Self)
    }

    fn next_u64(&mut self) -> std::io::Result<u64> {
        use std::io::Read;
        let mut bytes = [0u8; 8];
        self.0.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }
}

#[cfg(windows)]
#[link(name = "bcrypt")]
unsafe extern "system" {
    fn BCryptGenRandom(
        algorithm: *mut std::ffi::c_void,
        buffer: *mut u8,
        count: u32,
        flags: u32,
    ) -> i32;
}

#[cfg(windows)]
impl OsSecureRandom {
    fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    fn next_u64(&mut self) -> std::io::Result<u64> {
        let mut bytes = [0u8; 8];
        // BCRYPT_USE_SYSTEM_PREFERRED_RNG uses the Windows system CSPRNG.
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                0x0000_0002,
            )
        };
        if status < 0 {
            return Err(std::io::Error::other("Windows system CSPRNG unavailable"));
        }
        Ok(u64::from_le_bytes(bytes))
    }
}

impl OsSecureRandom {
    // Rejection sampling removes modulo bias, even for alphabet sizes that
    // do not divide 2^64. No use of floating-point arithmetic.
    fn index_below(&mut self, bound: u64) -> std::io::Result<u64> {
        if bound == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput, "random bound is zero",
            ));
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let sample = self.next_u64()?;
            if sample >= threshold {
                return Ok(sample % bound);
            }
        }
    }
}

const MAX_SECURE_RANDOM_CHARACTERS: usize = 4096;
const DEFAULT_PASSWORD_ALPHABET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}?";

fn checked_random_alphabet(alphabet: &str) -> Option<Vec<char>> {
    let mut seen = HashSet::new();
    let mut characters = Vec::new();
    for character in alphabet.chars() {
        if characters.len() == MAX_SECURE_RANDOM_CHARACTERS || !seen.insert(character) {
            return None;
        }
        characters.push(character);
    }
    (!characters.is_empty()).then_some(characters)
}

fn sample_secure_string(
    rng: &mut OsSecureRandom,
    length: usize,
    alphabet: &[char],
) -> std::io::Result<String> {
    let mut result = String::with_capacity(length.saturating_mul(4));
    let bound = alphabet.len() as u64;
    for _ in 0..length {
        result.push(alphabet[rng.index_below(bound)? as usize]);
    }
    Ok(result)
}

fn sample_secure_integer(
    rng: &mut OsSecureRandom,
    minimum: i64,
    maximum: i64,
) -> std::io::Result<i64> {
    let width = (i128::from(maximum) - i128::from(minimum)) as u128 + 1;
    let offset = if width == (1u128 << 64) {
        rng.next_u64()?
    } else {
        rng.index_below(width as u64)?
    };
    Ok((i128::from(minimum) + i128::from(offset)) as i64)
}
