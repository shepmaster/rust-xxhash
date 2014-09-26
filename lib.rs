#![crate_name="xxhash"]
#![crate_type="lib"]

#![allow(bad_style)]

#![feature(default_type_params, globs)]


#[cfg(test)]
extern crate test;

#[cfg(test)]
extern crate libc;

pub mod xxh32;
pub mod xxh64;

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
