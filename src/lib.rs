#![no_std]
#![feature(const_fn)]

#[allow(unused)]
#[macro_use]
extern crate typenum;

mod register;

pub mod bounds;
pub mod macros;
pub use crate::register::*;
