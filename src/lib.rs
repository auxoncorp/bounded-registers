//! # Registers
//!
//! A high-assurance register code generation and interaction library.
//!
//! ## Install
//!
//! ```not_rust
//! $ git clone git@github.com:auxoncorp/registers.git
//! $ cd registers && cargo install
//! ```
//!
//! ## Use
//!
//! There are two core pieces to `registers`:
//!
//! ### I. The macro
//!
//! ```
//! # #[macro_use]
//! # extern crate registers;
//! # #[macro_use]
//! # extern crate typenum;
//! register! {
//!     Status,
//!     u8,
//!     RW,
//!     Fields [
//!         On WIDTH(U1) OFFSET(U0),
//!         Dead WIDTH(U1) OFFSET(U1),
//!         Color WIDTH(U3) OFFSET(U2) [
//!             Red = U1,
//!             Blue = U2,
//!             Green = U3,
//!             Yellow = U4
//!         ]
//!     ]
//! }
//! # fn main() {}
//! ```
//!
//! The `register!` macro generates the code necessary for ergonomic
//! register access and manipulation. The expected input for the macro is
//! as follows:
//! 1. The register name.
//! 1. Its numeric type.
//! 1. Its mode, either `RO` (read only), `RW` (read write), or `WO`
//!    (write only).
//! 1. The register's fields, beginning with `Fields [`, and then a
//!    closing `]` at the end.
//!
//! A field constists of its name, its width, and its offset within the
//! register. Optionally, one may also state enum-like key/value pairs for
//! the values of the field, nested within the field declaration with
//! `[]`'s
//!
//! The code which this macro generates is a tree of nested modules where
//! the root is a module called `$register_name`. Within `$register_name`,
//! there will be the register itself, as `$register_name::Register`, as
//! well as a child module for each field.
//!
//! Within each field module, one can find the field itself, as
//! `$register_name::$field_name::Field`, as well as a few helpful aliases
//! and constants.
//!
//! * `$register_name::$field_name::Read`: In order to read a field, an
//!   instance of that field must be given to have access to its mask and
//!   offset. `Read` can be used as an argument to `get_field` so one does
//!   not have to construct an arbitrary one when doing a read.
//! * `$register_name::$field_name::Clear`: A field whose value is
//!   zero. Passing it to `modify` will clear that field in the register.
//! * `$register_name::$field_name::Set`: A field whose value is
//!   `$field_max`.  Passing it to `modify` will set that field to its max
//!   value in the register. This is useful particularly in the case of
//!   single-bit wide fields.
//! * `$register_name::$field_name::$enum_kvs`: constants mapping the enum
//!   like field names to values.
//!
//! ### II. Interacting with registers
//!
//! #### Through a constructor
//!
//! ```
//! # #[macro_use] // register! leans on typenum's op! macro.
//! # extern crate typenum;
//! # #[macro_use]
//! # extern crate registers;
//! # use registers::ReadWriteRegister;
//! # register! {
//! #     Status,
//! #     u8,
//! #     RW,
//! #     Fields [
//! #         On WIDTH(U1) OFFSET(U0),
//! #         Dead WIDTH(U1) OFFSET(U1),
//! #         Color WIDTH(U3) OFFSET(U2) [
//! #             Red = U1,
//! #             Blue = U2,
//! #             Green = U3,
//! #             Yellow = U4
//! #         ]
//! #     ]
//! # }
//! fn main() {
//!     let mut reg = Status::Register::new(0);
//!     reg.modify(Status::Dead::Set);
//!     assert_eq!(reg.read(), 2);
//! }
//! ```
//!
//! In this example, we initialize a register with the value `0` and then
//! set the `Dead` bit—the second field—which should produce the value `2`
//! when interpreting this word-sized register as a `u32`.
//!
//! #### Through a register block
//!
//! Here we take a known address, one we may find in a developer's manual,
//! and interpret that address as a register block. We can then
//! dereference that pointer and use the register API to access the
//! registers in the block.
//!
//! You can then implement `Deref` and `DerefMut` for a type which holds
//! onto the address of such a register block. This fills in the gaps for
//! method lookup (during typechecking) so that you can ergonomically use
//! this type to interact with the register block:
//!
//! ```
//! #[macro_use]
//! extern crate registers;
//! #[macro_use]
//! extern crate typenum;
//!
//! use core::ops::{Deref, DerefMut};
//!
//! use registers::{ReadOnlyRegister, ReadWriteRegister};
//!
//! register! {
//!     UartRX,
//!     u32,
//!     RO,
//!     Fields [
//!         Data        WIDTH(U8) OFFSET(U0),
//!         ParityError WIDTH(U1) OFFSET(U10),
//!         Brk         WIDTH(U1) OFFSET(U11),
//!         FrameError  WIDTH(U1) OFFSET(U12),
//!         Overrrun    WIDTH(U1) OFFSET(U13),
//!         Error       WIDTH(U1) OFFSET(U14),
//!         ChrRdy      WIDTH(U1) OFFSET(U15)
//!     ]
//! }
//!
//! register! {
//!     UartTX,
//!     u32,
//!     WO,
//!     Fields [
//!         Data WIDTH(U8) OFFSET(U0)
//!     ]
//! }
//!
//! register! {
//!     UartControl1,
//!     u32,
//!     RW,
//!     Fields [
//!         Enable              WIDTH(U1) OFFSET(U0),
//!         Doze                WIDTH(U1) OFFSET(U1),
//!         AgingDMATimerEnable WIDTH(U1) OFFSET(U2),
//!         TxRdyDMAENable      WIDTH(U1) OFFSET(U3),
//!         SendBreak           WIDTH(U1) OFFSET(U4),
//!         RTSDeltaInterrupt   WIDTH(U1) OFFSET(U5),
//!         TxEmptyInterrupt    WIDTH(U1) OFFSET(U6),
//!         Infrared            WIDTH(U1) OFFSET(U7),
//!         RecvReadyDMA        WIDTH(U1) OFFSET(U8),
//!         RecvReadyInterrupt  WIDTH(U1) OFFSET(U9),
//!         IdleCondition       WIDTH(U2) OFFSET(U10),
//!         IdleInterrupt       WIDTH(U1) OFFSET(U12),
//!         TxReadyInterrupt    WIDTH(U1) OFFSET(U13),
//!         AutoBaud            WIDTH(U1) OFFSET(U14),
//!         AutoBaudInterrupt   WIDTH(U1) OFFSET(U15)
//!     ]
//! }
//!
//! #[repr(C)]
//! pub struct UartBlock {
//!     rx: UartRX::Register,
//!     _padding1: [u32; 15],
//!     tx: UartTX::Register,
//!     _padding2: [u32; 15],
//!     control1: UartControl1::Register,
//! }
//!
//! pub struct Regs {
//!     addr: usize,
//! }
//!
//! impl Deref for Regs {
//!     type Target = UartBlock;
//!
//!     fn deref(&self) -> &UartBlock {
//!         unsafe { &*(self.addr as *const UartBlock) }
//!     }
//! }
//!
//! impl DerefMut for Regs {
//!     fn deref_mut(&mut self) -> &mut UartBlock {
//!         unsafe { &mut *(self.addr as *mut UartBlock) }
//!     }
//! }
//!
//! fn main() {
//!     let mut x = [0_u32; 33];
//!     let mut regs = Regs {
//!         // Some shenanigans to get at `x` as though it were a
//!         // pointer. Normally you'd be given some address like
//!         // `0xDEADBEEF` over which you'd instantiate a `Regs`.
//!         addr: &mut x as *mut [u32; 33] as usize,
//!     };
//!
//!     assert_eq!(regs.rx.read(), 0);
//!     regs.control1
//!         .modify(UartControl1::Enable::Set + UartControl1::RecvReadyInterrupt::Set);
//!
//!     // The first bit and the 10th bit should be set.
//!     assert_eq!(regs.control1.read(), 0b_10_0000_0001);
//! }
//! ```
//!
//! ## Theory
//!
//! `registers` employs values—specifically numbers—at the type-level in
//! order to get compile time assertions on interactions with a
//! register. Each field's width is used to determine a maximum value, and
//! then reading from and writing to those fields is either checked at
//! compile time, through the `checked` function, or is expected to
//! _carry_ a proof, which uses the aforementioned bound to construct a
//! value at runtime which is known to not contravene it.
#![no_std]
#![feature(const_fn)]

#[allow(unused)]
#[macro_use]
extern crate typenum;

pub mod bounds;
pub mod macros;

mod register;
pub use crate::register::*;
