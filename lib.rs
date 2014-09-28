#![crate_name="xxhash"]
#![crate_type="lib"]

#![allow(bad_style)]

#![feature(default_type_params, globs, macro_rules, phase)]


extern crate core;

#[cfg(test)] #[phase(plugin, link)] extern crate log;
#[cfg(test)] extern crate test;
#[cfg(test)] extern crate libc;

#[cfg(test)]
mod ffi {
    #![allow(dead_code)]

    pub use libc::{c_void, c_int, c_uint, c_longlong, c_ulonglong ,size_t};

    #[repr(C)]
    pub enum XXH_Endianess { 
        XXH_BigEndian=0, 
        XXH_LittleEndian=1 
    }

    #[cfg(target_endian="big")]
    pub static ENDIAN: XXH_Endianess = XXH_BigEndian;
    
    #[cfg(target_endian="little")]
    pub static ENDIAN: XXH_Endianess = XXH_LittleEndian;

    
    
    #[cfg(clang)]
    #[link(name="xxhash-clang")]
    extern {
        pub fn XXH64(input: *const c_void, len:c_int, seed: u64)-> c_longlong;
        pub fn XXH64_init(seed: u64) -> *mut c_void;
        pub fn XXH64_update(state: *mut c_void, input: *const c_void, len: c_int, endian: XXH_Endianess) -> bool;
        pub fn XXH64_digest(state: *mut c_void) -> u64;
        
        pub fn XXH32(input: *const c_void, len:c_int, seed: u32)-> c_uint;
        pub fn XXH32_init(seed: u32) -> *mut c_void;
        pub fn XXH32_update(state: *mut c_void, input: *const c_void, len: c_int, endian: XXH_Endianess) -> bool;
        pub fn XXH32_digest(state: *mut c_void) -> u32;
    }

    #[cfg(gcc)]
    #[link(name="xxhash-gcc")]
    extern {
        pub fn XXH64(input: *const c_void, len:c_int, seed: u64)-> c_longlong;
        pub fn XXH64_init(seed: u64) -> *mut c_void;
        pub fn XXH64_update(state: *mut c_void, input: *const c_void, len: c_uint, endian: XXH_Endianess) -> bool;
        pub fn XXH64_digest(state: *mut c_void) -> u64;
        
        pub fn XXH32(input: *const c_void, len:c_int, seed: u32)-> c_uint;
        pub fn XXH32_init(seed: u32) -> *mut c_void;
        pub fn XXH32_update(state: *mut c_void, input: *const c_void, len: c_int, endian: XXH_Endianess) -> bool;
        pub fn XXH32_digest(state: *mut c_void) -> u32;
    }
}

macro_rules! read_ptr(($p:ident, $size:ty) => ({
    let mut bp: *const $size = transmute($p);
    let data: $size = *bp;
    bp = bp.offset(1);
    $p = transmute(bp);
    data.to_le()
}))

pub mod xxh64 {
    use helper::*;
    use core::mem::{uninitialized,transmute};
    use core::raw::{Repr};
    use core::ptr::{copy_memory};
    
    static PRIME1: u64 = 11400714785074694791_u64;
    static PRIME2: u64 = 14029467366897019727_u64;
    static PRIME3: u64 =  1609587929392839161_u64;
    static PRIME4: u64 =  9650029242287828579_u64;
    static PRIME5: u64 =  2870177450012600261_u64;
    
    fn rotl64(x: u64, b: uint) -> u64 { #![inline(always)]
        ((x << b) | (x >> (64 - b)))
    }
    
    pub fn oneshot(input: &[u8], seed: u64) -> u64 {
        let mut state = State::new(seed);
        state.update(input);
        state.digest()
    }
        
    pub struct State {
        memory: [u64, ..4],
        v1: u64,
        v2: u64,
        v3: u64,
        v4: u64,
        total_len: u64,
        seed: u64,
        memsize: uint,
    }
    
    impl State {
        pub fn new(seed: u64) -> State { #![inline]
            let mut state: State = unsafe { uninitialized() };
            state.reset(seed);
            state
        }

        pub fn reset(&mut self, seed: u64) { #![inline]
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

            if self.memsize + rem < 32 {
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
                let bump: uint = 32 - self.memsize;
                copy_memory(dst, data, bump);
                let mut p: *const u8 = transmute(mem);

                macro_rules! read(($size:ty) => (read_ptr!(p, $size)))

                macro_rules! eat(($v: ident) => ({
                    $v += read!(u64) * PRIME2; $v = rotl64($v, 31); $v *= PRIME1;
                }))
                
                let mut v1: u64 = self.v1;
                let mut v2: u64 = self.v2;
                let mut v3: u64 = self.v3;
                let mut v4: u64 = self.v4;

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
                macro_rules! read(($size:ty) => (read_ptr!(data, $size)))

                macro_rules! eat(($v: ident) => ({
                    $v += read!(u64) * PRIME2; $v = rotl64($v, 31); $v *= PRIME1;
                }))
                
                let mut v1: u64 = self.v1;
                let mut v2: u64 = self.v2;
                let mut v3: u64 = self.v3;
                let mut v4: u64 = self.v4;

                while rem >= 32 {
                    eat!(v1); eat!(v2); eat!(v3); eat!(v4);
                    rem -= 32;
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
        
        pub fn digest(&self) -> u64 { #![inline] unsafe {
            let mut rem = self.memsize;
            let mut h64: u64 = if self.total_len < 32 {
                self.seed + PRIME5
            } else {
                let mut v1: u64 = self.v1;
                let mut v2: u64 = self.v2;
                let mut v3: u64 = self.v3;
                let mut v4: u64 = self.v4;
                
                let mut h = rotl64(v1, 1) + rotl64(v2, 7) + rotl64(v3, 12) + rotl64(v4, 18);

                macro_rules! permute(($v: ident) => ({
                    $v *= PRIME2; $v = rotl64($v, 31); $v *= PRIME1; h ^= $v; h = h * PRIME1 + PRIME4;
                }))                
                permute!(v1); permute!(v2); permute!(v3); permute!(v4);
                
                h
            };
            
            let mut p: *const u8 = transmute(&self.memory);
            macro_rules! read(($size:ty) => (read_ptr!(p, $size)))

            
            h64 += self.total_len as u64;

            while rem >= 8 {
                let mut k1: u64 = read!(u64) * PRIME2; k1 = rotl64(k1, 31); k1 *= PRIME1;
                h64 ^= k1; 
                h64 = rotl64(h64, 27) * PRIME1 + PRIME4;
                rem -= 8;
            }
            
            while rem >= 4 {
                h64 ^= read!(u32) as u64 * PRIME1;
                h64 = rotl64(h64, 23) * PRIME2 + PRIME3;
                rem -= 4;
            }
            
            while rem > 0 {
                h64 ^= read!(u8) as u64 * PRIME5;
                h64 = rotl64(h64, 11) * PRIME1;
                rem -= 1;
            }
            
            h64 ^= h64 >> 33;
            h64 *= PRIME2;
            h64 ^= h64 >> 29;
            h64 *= PRIME3;
            h64 ^= h64 >> 32;
            
            h64
        }}

    }

    #[test]
    fn test_oneshot() { 
        test64(oneshot)
    }
    
    #[test]
    fn test_state() {
        test64(|v, seed|{
            let mut state = State::new(seed);
            state.update(v);
            state.digest()
        })
    }
    
    #[bench]
    fn bench_oneshot(b: &mut Bencher) {
        bench_base(b, |v| { oneshot(v, 0) })
    }
}

pub mod xxh32 {
    use helper::*;
    use core::mem::{uninitialized,transmute};
    use core::raw::{Repr};
    use core::ptr::{copy_memory};
    use std::hash::{Hash, Hasher};

    fn rotl32(x: u32, b: uint) -> u32 { #![inline(always)]
        ((x << b) | (x >> (32 - b)))
    }

    static PRIME1: u32 = 2654435761;
    static PRIME2: u32 = 2246822519;
    static PRIME3: u32 = 3266489917;
    static PRIME4: u32 = 668265263;
    static PRIME5: u32 = 374761393;

    pub fn oneshot(input: &[u8], seed: u32) -> u32 {
        let mut state = State::new(seed);
        state.update(input);
        state.digest()
    }
    
    pub struct State {
        // field names match the C implementation
        memory: [u32, ..4],
        total_len: u64,
        v1: u32,
        v2: u32,
        v3: u32,
        v4: u32,
        memsize: uint,
        seed: u32,
    }

    impl State {
        pub fn new(seed: u32) -> State { #![inline]
            // no need to write it twice
            let mut state: State = unsafe { uninitialized() };
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

                macro_rules! read(($size:ty) => (read_ptr!(p, $size)))

                macro_rules! eat(($v: ident) => ({
                    $v += read!(u32) * PRIME2; $v = rotl32($v, 13); $v *= PRIME1;
                }))
                
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
                macro_rules! read(($size:ty) => (read_ptr!(data, $size)))

                macro_rules! eat(($v: ident) => ({
                    $v += read!(u32) * PRIME2; $v = rotl32($v, 13); $v *= PRIME1;
                }))
                
                let mut v1: u32 = self.v1;
                let mut v2: u32 = self.v2;
                let mut v3: u32 = self.v3;
                let mut v4: u32 = self.v4;

                while rem >= 16 {
                    eat!(v1); eat!(v2); eat!(v3); eat!(v4);
                    rem -= 16;
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
            macro_rules! read(($size:ty) => (read_ptr!(p, $size)))

            h32 += self.total_len as u32;

            while rem >= 4 {
                h32 += read!(u32) * PRIME3;
                h32 = rotl32(h32, 17) * PRIME4;
                rem -= 4;
            }
            
            while rem > 0 {
                h32 += read!(u8) as u32 * PRIME5;
                h32 = rotl32(h32, 11) * PRIME1;
                rem -= 1;
            }
            
            h32 ^= h32 >> 15;
            h32 *= PRIME2;
            h32 ^= h32 >> 13;
            h32 *= PRIME3;
            h32 ^= h32 >> 16;

            h32
        }}
    }

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

    impl Hasher<State> for XXHasher {
        fn hash<T: Hash<State>>(&self, value: &T) -> u64 {
            let mut state = State::new(self.seed);
            value.hash(&mut state);
            state.digest() as u64
        }
    }
    
    #[test]
    fn test_oneshot() { 
        test32(oneshot)
    }
    
    #[test]
    fn test_state() {
        test32(|v, seed|{
            let mut state = State::new(seed);
            state.update(v);
            state.digest()
        })
    }
    
    #[bench]
    fn bench_oneshot(b: &mut Bencher) {
        bench_base(b, |v| { oneshot(v, 0) as u64 })
    }
}

#[cfg(test)]
mod helper {
    pub use test::*;
    static BUFSIZE: uint = 101;
    static PRIME: u32 = 2654435761;

    fn test_vec() -> Vec<u8> { #![inline(always)]
        let mut random: u32 = PRIME;
        let mut buf: Vec<u8> = Vec::with_capacity(BUFSIZE);
        for _ in range(0, BUFSIZE) {
            buf.push((random >> 24) as u8);
            random *= random;
        }
        buf
    }
    
    pub fn test32(f: |&[u8], u32| -> u32) { #![inline(always)]
        let buf = test_vec();
        let test = |size: uint, seed: u32, expected: u32| {
            let result = f(buf.slice(0, size), seed);
            assert_eq!(result, expected);
        };

        test(1,                0,      0xB85CBEE5);
        test(1,                PRIME,  0xD5845D64);
        test(14,               0,      0xE5AA0AB4);
        test(14,               PRIME,  0x4481951D);
        test(BUFSIZE,          0,      0x1F1AA412);
        test(BUFSIZE,          PRIME,  0x498EC8E2);
    }
    
    pub fn test64(f: |&[u8], u64| -> u64) { #![inline(always)]
        let buf = test_vec();
        let test = |size: uint, seed: u64, expected: u64| {
            let result = f(buf.slice_to(size), seed);
            assert_eq!(result, expected);
        };

        test(1,                0,             0x4FCE394CC88952D8);
        test(1,                PRIME as u64,  0x739840CB819FA723);
        test(14,               0,             0xCFFA8DB881BC3A3D);
        test(14,               PRIME as u64,  0x5B9611585EFCC9CB);
        test(BUFSIZE,          0,             0x0EAB543384F878AD);
        test(BUFSIZE,          PRIME as u64,  0xCAA65939306F1E21);
    }
    
    
    pub fn bench_base(bench: &mut Bencher, f: |&[u8]| -> u64 ) { #![inline(always)]
        static BUFSIZE: uint = 1024*1024;

        let mut v: Vec<u8> = Vec::with_capacity(BUFSIZE);
        for i in range(0, BUFSIZE) {
            v.push(i as u8);
        }
        
        bench.iter( || f(v.as_slice()) );
        bench.bytes = BUFSIZE as u64;
    }
}
