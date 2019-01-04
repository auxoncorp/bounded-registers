#![no_std]
#![feature(const_fn)]

extern crate type_bounds;
#[macro_use]
extern crate typenum;

pub mod macros;
pub mod read_only;
pub mod read_write;
