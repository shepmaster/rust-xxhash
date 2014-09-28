#![crate_name="xxhash"]
#![crate_type="lib"]

#![allow(dead_assignment)] // macro

#![feature(default_type_params, globs, macro_rules, phase)]

#[cfg(test)] #[phase(plugin, link)] extern crate log;
#[cfg(test)] extern crate test;


pub mod xxh64 {
    use std::mem::{uninitialized,transmute};
    use std::raw::{Repr};
    use std::ptr::{copy_memory};
    use std::hash::{Writer,Hash,Hasher};
    use std::default::Default;
    
    #[cfg(test)] use test::*;
    
    // large prime, new_with_seed(0) is so boring
    static HAPPY_SEED: u64 = 18446744073709551557_u64;
    
    static PRIME1: u64 =     11400714785074694791_u64;
    static PRIME2: u64 =     14029467366897019727_u64;
    static PRIME3: u64 =      1609587929392839161_u64;
    static PRIME4: u64 =      9650029242287828579_u64;
    static PRIME5: u64 =      2870177450012600261_u64;
    
    fn rotl64(x: u64, b: uint) -> u64 { #![inline(always)]
        ((x << b) | (x >> (64 - b)))
    }
    
    // read an integer, advance the pointer by the appropriate amount
    // and do the endian dance
    macro_rules! read_ptr(($p:ident, $size:ty) => ({
        let mut bp: *const $size = transmute($p);
        let data: $size = *bp;
        bp = bp.offset(1);
        $p = transmute(bp);
        data.to_le()
    }))

    pub fn oneshot(input: &[u8], seed: u64) -> u64 {
        let mut state = XX64State::new_with_seed(seed);
        state.update(input);
        state.digest()
    }
    
    pub struct XX64State {
        memory: [u64, ..4],
        v1: u64,
        v2: u64,
        v3: u64,
        v4: u64,
        total_len: u64,
        seed: u64,
        memsize: uint,
    }
    
    impl XX64State {
        pub fn new_with_seed(seed: u64) -> XX64State { #![inline]
            let mut state: XX64State = unsafe { uninitialized() };
            state.reset(seed);
            state
        }

        pub fn new() -> XX64State { #![inline]
            XX64State::new_with_seed(HAPPY_SEED)
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

    impl Writer for XX64State {
        fn write(&mut self, msg: &[u8]) { #![inline]
            self.update(msg);
        }
    }
    
    impl Clone for XX64State {
        
        fn clone(&self) -> XX64State { #![inline]
            *self
        }
    }
    
    pub struct XX64Hasher {
        seed: u64
    }
    
    impl XX64Hasher {
        pub fn new() -> XX64Hasher { #![inline]
            XX64Hasher::new_with_seed(18446744073709551557u64)
        }
        
        pub fn new_with_seed(seed: u64) -> XX64Hasher { #![inline]
            XX64Hasher { seed: seed }
        }
    }
    
    impl Hasher<XX64State> for XX64Hasher {        
        fn hash<T: Hash<XX64State>>(&self, value: &T) -> u64 { #![inline]
            let mut state = XX64State::new_with_seed(self.seed);
            value.hash(&mut state);
            state.digest()
        }
    }

    impl Default for XX64Hasher {
        fn default() -> XX64Hasher { #![inline]
            XX64Hasher::new()
        }
    }

    pub fn hash<T: Hash<XX64State>>(value: &T) -> u64 { #![inline]
        let mut state = XX64State::new();
        value.hash(&mut state);
        state.digest()
    }

    pub fn hash_with_seed<T: Hash<XX64State>>(seed: u64, value: &T) -> u64 { #![inline]
        let mut state = XX64State::new_with_seed(seed);
        value.hash(&mut state);
        state.digest()
    }

    #[test]
    fn test_oneshot() {
        test_base(|v, seed|{
            let mut state = XX64State::new_with_seed(seed);
            state.update(v);
            state.digest()
        })
    }

    #[test]
    fn test_chunks() {
        test_base(|v, seed|{
            let mut state = XX64State::new_with_seed(seed);
            for chunk in v.chunks(15) {
                state.update(chunk);
            }
            state.digest()
        })
    }


    #[cfg(test)]
    fn test_base(f: |&[u8], u64| -> u64) { #![inline(always)]
        // the official sanity test
        static BUFSIZE: uint = 101;
        static PRIME: u32 = 2654435761;

        let mut random: u32 = PRIME;
        let mut buf: Vec<u8> = Vec::with_capacity(BUFSIZE);
        for _ in range(0, BUFSIZE) {
            buf.push((random >> 24) as u8);
            random *= random;
        }
        
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
    
    #[cfg(test)]
    fn bench_base(bench: &mut Bencher, f: |&[u8]| -> u64 ) { #![inline(always)]
        static BUFSIZE: uint = 64*1024;

        let mut v: Vec<u8> = Vec::with_capacity(BUFSIZE);
        for i in range(0, BUFSIZE) {
            v.push(i as u8);
        }
        
        bench.iter( || f(v.as_slice()) );
        bench.bytes = BUFSIZE as u64;
    }
    
    #[bench]
    fn bench_64k_oneshot(b: &mut Bencher) {
        bench_base(b, |v| { oneshot(v, 0) })
    }
    
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
}
