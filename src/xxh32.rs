use std::mem::{uninitialized,transmute};
use std::num::Int;
use std::raw::{Repr};
use std::ptr::{copy_memory};
use std::hash::{Hash, Hasher, Writer};
use std::default::Default;

#[cfg(test)] use test::Bencher;

fn rotl32(x: u32, b: uint) -> u32 { #![inline(always)]
    ((x << b) | (x >> (32 - b)))
}

static PRIME1: u32 = 2654435761;
static PRIME2: u32 = 2246822519;
static PRIME3: u32 = 3266489917;
static PRIME4: u32 = 668265263;
static PRIME5: u32 = 374761393;

pub fn oneshot(input: &[u8], seed: u32) -> u32 {
    let mut state = XXState::new_with_seed(seed);
    state.update(input);
    state.digest()
}

#[deriving(Copy)]
pub struct XXState {
    // field names match the C implementation
    memory: [u32; 4],
    total_len: u64,
    v1: u32,
    v2: u32,
    v3: u32,
    v4: u32,
    memsize: uint,
    seed: u32,
}

impl XXState {
    pub fn new_with_seed(seed: u32) -> XXState { #![inline]
        // no need to write it twice
        let mut state: XXState = unsafe { uninitialized() };
        state.reset(seed);
        state
    }

    pub fn reset(&mut self, seed: u32) { #![inline]
        self.seed = seed;
        self.v1 = seed + PRIME1 + PRIME2;
        self.v2 = seed + PRIME2;
        self.v3 = seed;
        self.v4 = seed - PRIME1;
        self.total_len = 0;
        self.memsize = 0;
    }

    pub fn update(&mut self, input: &[u8]) { #![inline] unsafe {
        let mem: *mut u8 = transmute(&self.memory);
        let mut rem: uint = input.len();
        let mut data: *const u8 = input.repr().data;

        self.total_len += rem as u64;

        if self.memsize + rem < 16 {
            // not enough data for one 32-byte chunk, so just fill the buffer and return.
            let dst: *mut u8 = mem.offset(self.memsize as int);
            copy_memory(dst, data, rem);
            self.memsize += rem;
            return;
        }

        if self.memsize != 0 {
            // some data left from previous update
            // fill the buffer and eat it
            let dst: *mut u8 = mem.offset(self.memsize as int);
            let bump: uint = 16 - self.memsize;
            copy_memory(dst, data, bump);
            let mut p: *const u8 = transmute(mem);
            let mut r: uint = 32;

            macro_rules! read(() => (read_ptr!(p, r, u32)));

            macro_rules! eat(($v: ident) => ({
                $v += read!() * PRIME2; $v = rotl32($v, 13); $v *= PRIME1;
            }));

            let mut v1: u32 = self.v1;
            let mut v2: u32 = self.v2;
            let mut v3: u32 = self.v3;
            let mut v4: u32 = self.v4;

            eat!(v1); eat!(v2); eat!(v3); eat!(v4);

            self.v1 = v1;
            self.v2 = v2;
            self.v3 = v3;
            self.v4 = v4;

            data = data.offset(bump as int);
            rem -= bump;
            self.memsize = 0;
        }

        {
            macro_rules! read(() => (read_ptr!(data, rem, u32)));

            macro_rules! eat(($v: ident) => ({
                $v += read!() * PRIME2; $v = rotl32($v, 13); $v *= PRIME1;
            }));

            let mut v1: u32 = self.v1;
            let mut v2: u32 = self.v2;
            let mut v3: u32 = self.v3;
            let mut v4: u32 = self.v4;

            while rem >= 16 {
                eat!(v1); eat!(v2); eat!(v3); eat!(v4);
            }

            self.v1 = v1;
            self.v2 = v2;
            self.v3 = v3;
            self.v4 = v4;
        }

        if rem > 0 {
            copy_memory(mem, data, rem);
            self.memsize = rem;
        }
    }}

    /// Can be called on intermediate states
    pub fn digest(&self) -> u32 { #![inline] unsafe {
        let mut rem = self.memsize;
        let mut h32: u32 = if self.total_len < 16 {
            self.seed + PRIME5
        } else {
            rotl32(self.v1, 1) + rotl32(self.v2, 7) + rotl32(self.v3, 12) + rotl32(self.v4, 18)
        };

        let mut p: *const u8 = transmute(&self.memory);
        macro_rules! read(($size:ty) => (read_ptr!(p, rem, $size) as u32));

        h32 += self.total_len as u32;

        while rem >= 4 {
            h32 += read!(u32) * PRIME3;
            h32 = rotl32(h32, 17) * PRIME4;
        }

        while rem > 0 {
            h32 += read!(u8) * PRIME5;
            h32 = rotl32(h32, 11) * PRIME1;
        }

        h32 ^= h32 >> 15;
        h32 *= PRIME2;
        h32 ^= h32 >> 13;
        h32 *= PRIME3;
        h32 ^= h32 >> 16;

        h32
    }}
}

#[deriving(Copy)]
pub struct XXHasher {
    seed: u32
}

impl XXHasher {
    pub fn new() -> XXHasher { #![inline]
        XXHasher::new_with_seed(0)
    }

    pub fn new_with_seed(seed: u32) -> XXHasher { #![inline]
        XXHasher { seed: seed }
    }
}

impl Hasher<XXState> for XXHasher {
    fn hash<Sized? T: Hash<XXState>>(&self, value: &T) -> u64 {
        let mut state = XXState::new_with_seed(self.seed);
        value.hash(&mut state);
        state.digest() as u64
    }
}

impl Writer for XXState {
    fn write(&mut self, msg: &[u8]) { #![inline]
        self.update(msg);
    }
}

impl Clone for XXState {
    fn clone(&self) -> XXState { #![inline]
        *self
    }
}

impl Default for XXHasher {
    fn default() -> XXHasher { #![inline]
        XXHasher::new()
    }
}

pub fn hash<T: Hash<XXState>>(value: &T) -> u64 { #![inline]
    let mut state = XXState::new_with_seed(0);
    value.hash(&mut state);
    state.digest() as u64
}

pub fn hash_with_seed<T: Hash<XXState>>(seed: u64, value: &T) -> u64 { #![inline]
    let mut state = XXState::new_with_seed(seed as u32);
    value.hash(&mut state);
    state.digest() as u64
}

/// the official sanity test
#[cfg(test)]
fn test_base(f: |&[u8], u32| -> u32) { #![inline(always)]
    static BUFSIZE: uint = 101;
    static PRIME: u32 = 2654435761;

    let mut random: u32 = PRIME;
    let mut buf: Vec<u8> = Vec::with_capacity(BUFSIZE);
    for _ in range(0, BUFSIZE) {
        buf.push((random >> 24) as u8);
        random *= random;
    }

    let test = |size: uint, seed: u32, expected: u32| {
        let result = f(buf.slice_to(size), seed);
        assert_eq!(result, expected);
    };


    test(1,                0,      0xB85CBEE5);
    test(1,                PRIME,  0xD5845D64);
    test(14,               0,      0xE5AA0AB4);
    test(14,               PRIME,  0x4481951D);
    test(BUFSIZE,          0,      0x1F1AA412);
    test(BUFSIZE,          PRIME,  0x498EC8E2);
}

#[cfg(test)]
fn bench_base(bench: &mut Bencher, f: |&[u8]| -> u32 ) { #![inline(always)]
    static BUFSIZE: uint = 64*1024;

    let mut v: Vec<u8> = Vec::with_capacity(BUFSIZE);
    for i in range(0, BUFSIZE) {
        v.push(i as u8);
    }

    bench.iter( || f(v.as_slice()) );
    bench.bytes = BUFSIZE as u64;
}

#[test]
fn test_oneshot() {
    test_base(|v, seed|{
        let mut state = XXState::new_with_seed(seed);
        state.update(v);
        state.digest()
    })
}

#[test]
fn test_chunks() {
    test_base(|v, seed|{
        let mut state = XXState::new_with_seed(seed);
        for chunk in v.chunks(15) {
            state.update(chunk);
        }
        state.digest()
    })
}

#[bench]
fn bench_64k_oneshot(b: &mut Bencher) {
    bench_base(b, |v| { oneshot(v, 0) })
}

/*
    * The following tests match those of SipHash.
    */


#[test] #[cfg(target_arch = "arm")]
fn test_hash_uint() {
    let val = 0xdeadbeef_deadbeef_u64;
    assert!(hash(&(val as u64)) != hash(&(val as uint)));
    assert_eq!(hash(&(val as u32)), hash(&(val as uint)));
}
#[test] #[cfg(target_arch = "x86_64")]
fn test_hash_uint() {
    let val = 0xdeadbeef_deadbeef_u64;
    assert_eq!(hash(&(val as u64)), hash(&(val as uint)));
    assert!(hash(&(val as u32)) != hash(&(val as uint)));
}
#[test] #[cfg(target_arch = "x86")]
fn test_hash_uint() {
    let val = 0xdeadbeef_deadbeef_u64;
    assert!(hash(&(val as u64)) != hash(&(val as uint)));
    assert_eq!(hash(&(val as u32)), hash(&(val as uint)));
}

#[test]
fn test_hash_idempotent() {
    let val64 = 0xdeadbeef_deadbeef_u64;
    assert_eq!(hash(&val64), hash(&val64));
    let val32 = 0xdeadbeef_u32;
    assert_eq!(hash(&val32), hash(&val32));
}

#[test]
fn test_hash_no_bytes_dropped_64() {
    let val = 0xdeadbeef_deadbeef_u64;

    assert!(hash(&val) != hash(&zero_byte(val, 0)));
    assert!(hash(&val) != hash(&zero_byte(val, 1)));
    assert!(hash(&val) != hash(&zero_byte(val, 2)));
    assert!(hash(&val) != hash(&zero_byte(val, 3)));
    assert!(hash(&val) != hash(&zero_byte(val, 4)));
    assert!(hash(&val) != hash(&zero_byte(val, 5)));
    assert!(hash(&val) != hash(&zero_byte(val, 6)));
    assert!(hash(&val) != hash(&zero_byte(val, 7)));

    fn zero_byte(val: u64, byte: uint) -> u64 {
        assert!(byte < 8);
        val & !(0xff << (byte * 8))
    }
}

#[test]
fn test_hash_no_bytes_dropped_32() {
    let val = 0xdeadbeef_u32;

    assert!(hash(&val) != hash(&zero_byte(val, 0)));
    assert!(hash(&val) != hash(&zero_byte(val, 1)));
    assert!(hash(&val) != hash(&zero_byte(val, 2)));
    assert!(hash(&val) != hash(&zero_byte(val, 3)));

    fn zero_byte(val: u32, byte: uint) -> u32 {
        assert!(byte < 4);
        val & !(0xff << (byte * 8))
    }
}

#[test]
fn test_hash_no_concat_alias() {
    let s = ("aa", "bb");
    let t = ("aabb", "");
    let u = ("a", "abb");

    assert!(s != t && t != u);
    assert!(hash(&s) != hash(&t) && hash(&s) != hash(&u));

    let v: (&[u8], &[u8], &[u8]) = (&[1u8], &[0u8, 0], &[0u8]);
    let w: (&[u8], &[u8], &[u8]) = (&[1u8, 0, 0, 0], &[], &[]);

    assert!(v != w);
    assert!(hash(&v) != hash(&w));
}

#[bench]
fn bench_str_under_8_bytes(b: &mut Bencher) {
    let s = "foo";
    b.bytes=s.len() as u64;
    b.iter(|| {
        hash(&s)
    })
}

#[bench]
fn bench_str_of_8_bytes(b: &mut Bencher) {
    let s = "foobar78";
    b.bytes = s.len() as u64;
    b.iter(|| {
        hash(&s);
    })
}

#[bench]
fn bench_str_over_8_bytes(b: &mut Bencher) {
    let s = "foobarbaz0";
    b.bytes = s.len() as u64;
    b.iter(|| {
        hash(&s)
    })
}

#[bench]
fn bench_long_str(b: &mut Bencher) {
    let s = "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor \
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute \
irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui \
officia deserunt mollit anim id est laborum.";
    b.bytes = s.len() as u64;
    b.iter(|| {
        hash(&s)
    })
}

#[bench]
fn bench_u64(b: &mut Bencher) {
    let u = 16262950014981195938u64;
    b.bytes = 8;
    b.iter(|| {
        hash(&u)
    })
}
