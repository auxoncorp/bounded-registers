#![no_std]
#![feature(const_fn)]

#[allow(unused)]
#[macro_use]
extern crate typenum;

pub mod bounds;
pub mod macros;

mod register;
pub use crate::register::*;
