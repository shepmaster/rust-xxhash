#![crate_name="xxhash"]
#![crate_type="lib"]

#![deny(warnings)]
#![allow(bad_style)]

#![feature(default_type_params)]

#[cfg(test)]
extern crate test;

#[cfg(test)]
extern crate libc;

pub mod xxh32;
pub mod xxh64;
