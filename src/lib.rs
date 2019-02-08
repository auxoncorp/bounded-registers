#![no_std]
#![feature(const_fn)]

extern crate type_bounds;

#[allow(unused)]
#[macro_use]
extern crate typenum;

mod register;

pub mod macros;
pub use crate::register::*;
