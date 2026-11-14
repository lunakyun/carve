//! MD5, SHA-1, SHA-256 and SHA-512 for the `hashed with` expression.
//! Small and dependency free. The SHA constants are computed once from
//! prime roots with exact integer math, so no big tables are typed in.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlg {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

pub fn digest_hex(alg: HashAlg, data: &[u8]) -> String {
    let bytes = match alg {
        HashAlg::Md5 => md5(data),
        HashAlg::Sha1 => sha1(data),
        HashAlg::Sha256 => sha256(data),
        HashAlg::Sha512 => sha512(data),
    };
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// `md5`, `sha-1`, `sha256`, ... in any case, with or without the dash.
pub fn alg_by_name(name: &str) -> Option<HashAlg> {
    match name.to_ascii_lowercase().replace('-', "").as_str() {
        "md5" => Some(HashAlg::Md5),
        "sha1" => Some(HashAlg::Sha1),
        "sha256" => Some(HashAlg::Sha256),
        "sha512" => Some(HashAlg::Sha512),
        _ => None,
    }
}

/// Pad like MD5/SHA: 0x80, zeros, then the bit length. `block` is 64 or 128.
fn pad(data: &[u8], block: usize, big_endian: bool) -> Vec<u8> {
    let len_bytes = if block == 128 { 16 } else { 8 };
    let mut v = data.to_vec();
    v.push(0x80);
    while v.len() % block != block - len_bytes {
        v.push(0);
    }
    let bits = (data.len() as u128) * 8;
    if big_endian {
        v.extend_from_slice(&bits.to_be_bytes()[16 - len_bytes..]);
    } else {
        v.extend_from_slice(&bits.to_le_bytes()[..len_bytes]);
    }
    v
}

fn md5(data: &[u8]) -> Vec<u8> {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15,
        21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    static K: OnceLock<[u32; 64]> = OnceLock::new();
    let k = K.get_or_init(|| {
        let mut k = [0u32; 64];
        for (i, x) in k.iter_mut().enumerate() {
            *x = (((i + 1) as f64).sin().abs() * 4_294_967_296.0) as u32;
        }
        k
    });
    let (mut a0, mut b0, mut c0, mut d0) = (0x6745_2301u32, 0xefcd_ab89u32, 0x98ba_dcfeu32, 0x1032_5476u32);
    for chunk in pad(data, 64, false).chunks(64) {
        let m: Vec<u32> = chunk
            .chunks(4)
            .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
            .collect();
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(k[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(S[i]));
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }
    [a0, b0, c0, d0].iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn sha1(data: &[u8]) -> Vec<u8> {
    let mut h: [u32; 5] = [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];
    for chunk in pad(data, 64, true).chunks(64) {
        let mut w = [0u32; 80];
        for (i, x) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([x[0], x[1], x[2], x[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i / 20 {
                0 => ((b & c) | (!b & d), 0x5A82_7999u32),
                1 => (b ^ c ^ d, 0x6ED9_EBA1),
                2 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let t = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e]) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().flat_map(|x| x.to_be_bytes()).collect()
}

// ---- exact integer roots for the SHA-2 constants ----

/// Little-endian 32-bit limbs.
type Big = Vec<u32>;

fn big_mul(a: &Big, b: &Big) -> Big {
    let mut out = vec![0u32; a.len() + b.len()];
    for (i, &x) in a.iter().enumerate() {
        let mut carry = 0u64;
        for (j, &y) in b.iter().enumerate() {
            let cur = u64::from(out[i + j]) + u64::from(x) * u64::from(y) + carry;
            out[i + j] = cur as u32;
            carry = cur >> 32;
        }
        out[i + b.len()] = carry as u32;
    }
    out
}

fn big_le(a: &Big, b: &Big) -> bool {
    let n = a.len().max(b.len());
    for i in (0..n).rev() {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x < y;
        }
    }
    true
}

fn from_u128(x: u128) -> Big {
    (0..4).map(|i| (x >> (32 * i)) as u32).collect()
}

/// floor(root(p * 2^(64*root))) for root 2 or 3. The result is below 2^70.
fn int_root(p: u64, root: u32) -> u128 {
    // target = p << (64 * root)
    let mut target: Big = vec![0; 2 * root as usize];
    target.push(p as u32);
    let mut x: u128 = 0;
    for bit in (0..70).rev() {
        let cand = x | (1u128 << bit);
        let c = from_u128(cand);
        let sq = big_mul(&c, &c);
        let pw = if root == 3 { big_mul(&sq, &c) } else { sq };
        if big_le(&pw, &target) {
            x = cand;
        }
    }
    x
}

fn primes(n: usize) -> Vec<u64> {
    let mut v: Vec<u64> = Vec::new();
    let mut c = 2u64;
    while v.len() < n {
        if v.iter().all(|p| !c.is_multiple_of(*p)) {
            v.push(c);
        }
        c += 1;
    }
    v
}

struct Consts {
    k512: [u64; 80],
    h512: [u64; 8],
}

fn consts() -> &'static Consts {
    static C: OnceLock<Consts> = OnceLock::new();
    C.get_or_init(|| {
        let ps = primes(80);
        let mut k512 = [0u64; 80];
        for (i, p) in ps.iter().enumerate() {
            k512[i] = int_root(*p, 3) as u64;
        }
        let mut h512 = [0u64; 8];
        for (i, p) in ps.iter().take(8).enumerate() {
            h512[i] = int_root(*p, 2) as u64;
        }
        Consts { k512, h512 }
    })
}

fn sha256(data: &[u8]) -> Vec<u8> {
    let c = consts();
    let k: Vec<u32> = c.k512.iter().take(64).map(|x| (x >> 32) as u32).collect();
    let mut h: [u32; 8] = std::array::from_fn(|i| (c.h512[i] >> 32) as u32);
    for chunk in pad(data, 64, true).chunks(64) {
        let mut w = [0u32; 64];
        for (i, x) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([x[0], x[1], x[2], x[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (x, y) in h.iter_mut().zip(v) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().flat_map(|x| x.to_be_bytes()).collect()
}

fn sha512(data: &[u8]) -> Vec<u8> {
    let c = consts();
    let mut h = c.h512;
    for chunk in pad(data, 128, true).chunks(128) {
        let mut w = [0u64; 80];
        for (i, x) in chunk.chunks(8).enumerate() {
            w[i] = u64::from_be_bytes([x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]]);
        }
        for i in 16..80 {
            let s0 = w[i - 15].rotate_right(1) ^ w[i - 15].rotate_right(8) ^ (w[i - 15] >> 7);
            let s1 = w[i - 2].rotate_right(19) ^ w[i - 2].rotate_right(61) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..80 {
            let s1 = v[4].rotate_right(14) ^ v[4].rotate_right(18) ^ v[4].rotate_right(41);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(c.k512[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(28) ^ v[0].rotate_right(34) ^ v[0].rotate_right(39);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (x, y) in h.iter_mut().zip(v) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().flat_map(|x| x.to_be_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_digests() {
        assert_eq!(digest_hex(HashAlg::Md5, b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            digest_hex(HashAlg::Md5, b"abc"),
            "900150983cd24fb0d6963f7d28e17f72"
        );
        assert_eq!(
            digest_hex(HashAlg::Sha1, b"abc"),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            digest_hex(HashAlg::Sha256, b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            digest_hex(HashAlg::Sha512, b"abc"),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }
}
