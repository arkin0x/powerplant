// pow.rs

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize)]
pub struct PowRequest {
    pub action: Vec<u8>,
    pub nonce_bounds: (usize, usize),
    pub nonce_start: u64,
    pub nonce_end: u64,
    pub target_pow: u32,
}

#[derive(Serialize, Deserialize)]
pub struct PowResponse {
    pub action: Vec<u8>,
    pub nonce: u64,
    pub pow: u32,
}

pub fn perform_pow(request: &PowRequest) -> PowResponse {
    let mut action = request.action.clone();
    let (start, end) = request.nonce_bounds;

    for nonce in request.nonce_start..=request.nonce_end {
        set_nonce_buffer(&mut action, start, end, nonce);
        let digest = Sha256::digest(&action);
        let pow = count_leading_zeroes_bin(&digest);

        if pow >= request.target_pow {
            return PowResponse {
                action,
                nonce,
                pow,
            };
        }
    }

    PowResponse {
        action: request.action.clone(),
        nonce: request.nonce_end,
        pow: 0,
    }
}

fn set_nonce_buffer(buffer: &mut [u8], start: usize, end: usize, nonce: u64) {
    for i in (start..end).rev() {
        buffer[i] = ((nonce & 0xF) as u8) + 48;
        nonce >>= 4;
    }
}

fn count_leading_zeroes_bin(digest: &[u8]) -> u32 {
    let mut count = 0;
    for &byte in digest {
        if byte == 0 {
            count += 8;
        } else {
            count += byte.leading_zeros();
            break;
        }
    }
    count
}
