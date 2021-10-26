# Bounded Registers

## Overview

A high-assurance memory-mapped register code generation and
interaction library.

## Getting Started

```shell
$ git clone git@github.com:auxoncorp/bounded-registers.git
$ cd bounded-registers && cargo +nightly build
```

## Usage

### The macro

```rust
register! {
    Status,
    u8,
    RW,
    Fields [
        On    WIDTH(U1) OFFSET(U0),
        Dead  WIDTH(U1) OFFSET(U1),
        Color WIDTH(U3) OFFSET(U2) [
            Red = U1,
            Blue = U2,
            Green = U3,
            Yellow = U4
        ]
    ]
}
```

The `register!` macro generates the code necessary for ergonomic
register access and manipulation. The expected input for the macro is
as follows:
1. The register name.
1. Its numeric type.
1. Its mode, either `RO` (read only), `RW` (read write), or `WO`
   (write only).
1. The register's fields, beginning with `Fields [`, and then a
   closing `]` at the end.

A field constists of its name, its width, and its offset within the
register. Optionally, one may also state enum-like key/value pairs for
the values of the field, nested within the field declaration with
`[]`'s

The code which this macro generates is a tree of nested modules where
the root is a module called `$register_name`. Within `$register_name`,
there will be the register itself, as `$register_name::Register`, as
well as a child module for each field.

Within each field module, one can find the field itself, as
`$register_name::$field_name::Field`, as well as a few helpful aliases
and constants.

- `$register_name::$field_name::Read`: In order to read a field, an
  instance of that field must be given to have access to its mask and
  offset. `Read` can be used as an argument to `get_field` so one does
  not have to construct an arbitrary one when doing a read.
- `$register_name::$field_name::Clear`: A field whose value is
  zero. Passing it to `modify` will clear that field in the register.
- `$register_name::$field_name::Set`: A field whose value is
  `$field_max`.  Passing it to `modify` will set that field to its max
  value in the register. This is useful particularly in the case of
  single-bit wide fields.
- `$register_name::$field_name::$enum_kvs`: constants mapping the enum
  like field names to values.

### Interacting with registers

#### Through a constructor

```rust
fn main() {
    let mut reg = Status::Register::new(0);
    reg.modify(Status::Dead::Set);
    assert_eq!(reg.read(), 2);
}
```

In this example, we initialize a register with the value `0` and then
set the `Dead` bit—the second field—which should produce the value `2`
when interpreting this word-sized register as a `u32`.

#### Through a register block

Here we take a known address, one we may find in a developer's manual,
and interpret that address as a register block. We can then
dereference that pointer and use the register API to access the
registers in the block.

```rust
register! {
    UartRX,
    RO,
    Fields [
        Data        WIDTH(U8) OFFSET(U0),
        ParityError WIDTH(U1) OFFSET(U10),
        Brk         WIDTH(U1) OFFSET(U11),
        FrameError  WIDTH(U1) OFFSET(U12),
        Overrrun    WIDTH(U1) OFFSET(U13),
        Error       WIDTH(U1) OFFSET(U14),
        ChrRdy      WIDTH(U1) OFFSET(U15)
    ]
}

register! {
    UartTX,
    WO,
    Fields [
        Data WIDTH(U8) OFFSET(U0)
    ]
}

register! {
    UartControl1,
    RW,
    Fields [
        Enable              WIDTH(U1) OFFSET(U0),
        Doze                WIDTH(U1) OFFSET(U1),
        AgingDMATimerEnable WIDTH(U1) OFFSET(U2),
        TxRdyDMAENable      WIDTH(U1) OFFSET(U3),
        SendBreak           WIDTH(U1) OFFSET(U4),
        RTSDeltaInterrupt   WIDTH(U1) OFFSET(U5),
        TxEmptyInterrupt    WIDTH(U1) OFFSET(U6),
        Infrared            WIDTH(U1) OFFSET(U7),
        RecvReadyDMA        WIDTH(U1) OFFSET(U8),
        RecvReadyInterrupt  WIDTH(U1) OFFSET(U9),
        IdleCondition       WIDTH(U2) OFFSET(U10),
        IdleInterrupt       WIDTH(U1) OFFSET(U12),
        TxReadyInterrupt    WIDTH(U1) OFFSET(U13),
        AutoBaud            WIDTH(U1) OFFSET(U14),
        AutoBaudInterrupt   WIDTH(U1) OFFSET(U15)
    ]
}
```

You can then implement `Deref` and `DerefMut` for a type which holds
onto the address of such a register block. This fills in the gaps for
method lookup (during typechecking) so that you can ergonomically use
this type to interact with the register block:

```rust
#[repr(C)]
pub struct UartBlock {
    rx: UartRX::Register,
    _padding1: [u32; 15],
    tx: UartTX::Register,
    _padding2: [u32; 15],
    control1: UartControl1::Register,
}

pub struct Regs {
    addr: usize,
}

impl Deref for Regs {
    type Target = UartBlock;

    fn deref(&self) -> &UartBlock {
        unsafe { &*(self.addr as *const UartBlock) }
    }
}

impl DerefMut for Regs {
    fn deref_mut(&mut self) -> &mut UartBlock {
        unsafe { &mut *(self.addr as *mut UartBlock) }
    }
}

fn main() {
    // A pretend register block.
    let mut x = [0_u32; 33];

    let mut regs = Regs {
        // Some shenanigans to get at `x` as though it were a
        // pointer. Normally you'd be given some address like
        // `0xDEADBEEF` over which you'd instantiate a `Regs`.
        addr: &mut x as *mut [u32; 33] as usize,
    };

    assert_eq!(regs.rx.read(), 0);

    regs.control1
        .modify(UartControl1::Enable::Set + UartControl1::RecvReadyInterrupt::Set);

    // The first bit and the 10th bit should be set.
    assert_eq!(regs.control1.read(), 0b_10_0000_0001);
}
```

### The Register API

The register API code is generated with docs, but you'll have to build
the rustdoc documentation for your library that uses
`bounded-registers` to be able to see it. For convenience, I've
extrapolated it here:

```rust
  impl Register {
      /// `new` constructs a read-write register around the
      /// given pointer.
      pub fn new(init: Width) -> Self;

      /// `get_field` takes a field and sets the value of that
      /// field to its value in the register.
      pub fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
          &self,
          f: F<Width, M, O, U, Register>,
      ) -> Option<F<Width, M, O, U, Register>>
      where
          U: IsGreater<U0, Output = True> + ReifyTo<Width>,
          M: ReifyTo<Width>,
          O: ReifyTo<Width>,
          U0: ReifyTo<Width>;

      /// `read` returns the current state of the register as a `Width`.
      pub fn read(&self) -> Width;

      /// `extract` pulls the state of a register out into a wrapped
      /// read-only register.
      pub fn extract(&self) -> ReadOnlyCopy<Width, Register>;

      /// `is_set` takes a field and returns true if that field's value
      /// is equal to its upper bound or not. This is of particular use
      /// in single-bit fields.
      pub fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(
          &self,
          f: F<Width, M, O, U, Register>,
      ) -> bool
      where
          U: IsGreater<U0, Output = True>,
          U: ReifyTo<Width>,
          M: ReifyTo<Width>,
          O: ReifyTo<Width>;

      // `Positioned` is a special trait that all fields implement, as
      // well as a type used as an accumulator when reading from or
      // writing to multiple fields. To use these functions with
      // multiple fields, join them together with `+`. An `Add`
      // implementation for fields has been provided for this purpose.

      /// `matches_any` returns whether or not any of the given fields
      /// match those fields values inside the register.
      pub fn matches_any<V: Positioned<Width = Width>>(&self, val: V) -> bool;

      /// `matches_all` returns whether or not all of the given fields
      /// match those fields values inside the register.
      pub fn matches_all<V: Positioned<Width = Width>>(&self, val: V) -> bool;

      /// `modify` takes one or more fields, joined by `+`, and
      /// sets those fields in the register, leaving the others
      /// as they were.
      pub fn modify<V: Positioned<Width = Width>>(&mut self, val: V);

      /// `write` sets the value of the whole register to the
      /// given `Width` value.
      pub unsafe fn write(&mut self, val: Width);
  }
```

## Theory

`bounded-registers` employs values—specifically numbers—at the type-level in
order to get compile time assertions on interactions with a
register. Each field's width is used to determine a maximum value, and
then reading from and writing to those fields is either checked at
compile time, through the `checked` function, or is expected to
/carry/ a proof, which uses the aforementioned bound to construct a
value at runtime which is known to not contravene it.

## License

`bounded-registers` is licensed under the MIT License (MIT) unless
otherwise noted. Please see [LICENSE](./LICENSE) for more details.
