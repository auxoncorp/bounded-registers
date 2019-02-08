/// The `register!` macro generates the code necessary for ergonomic register
/// access and manipulation. It is the crux of this crate. The expected input
/// for the macro is as follows:
/// 1. The register name.
/// 2. Its mode, either `RO` (read only), `RW` (read write), or `WO` (write
///    only).
/// 3. The register's fields, beginning with `Fields [`, and then a
///    closing `]` at the end.
///
/// A field constists of its name, its width, and its offset within the
/// register. Optionally, one may also state enum-like key/value pairs for the
/// values of the field, nested within the field declaration with `[]`'s
///
/// The code which this macro generates is a tree of nested modules where the
/// root is a module called `$register_name`. Within `$register_name`, there
/// will be the register itself, as `$register_name::Register`, as well as a
/// child module for each field.
///
/// Within each field module, one can find the field itself, as
/// `$register_name::$field_name::Field`, as well as a few helpful aliases and
/// constants.
///
/// * `$register_name::$field_name::Read`: In order to read a field, an instance
///   of that field must be given to have access to its mask and offset. `Read`
///   can be used as an argument to `get_field` so one does not have to
///   construct an arbitrary one when doing a read.
/// * `$register_name::$field_name::Clear`: A field whose value is zero. Passing
///   it to `modify` will clear that field in the register.
/// * `$register_name::$field_name::Set`: A field whose value is `$field_max`.
///   Passing it to `modify` will set that field to its max value in the
///   register. This is useful particularly in the case of single-bit wide
///   fields.
/// * `$register_name::$field_name::$enum_kvs`: constants mapping the enum like
///   field names to values.
///
/// An example register and its use is below:
/// ```
/// #[macro_use]
/// extern crate typenum;
/// #[macro_use]
/// extern crate registers;
///
/// use typenum::consts::U1;
///
/// use registers::ReadWriteRegister;
///
/// register! {
///     Status,
///     RW,
///     Fields [
///         On WIDTH(U1) OFFSET(U0),
///         Dead WIDTH(U1) OFFSET(U1),
///         Color WIDTH(U3) OFFSET(U2) [
///             Red = U1,
///             Blue = U2,
///             Green = U3,
///             Yellow = U4
///         ]
///     ]
/// }
///
/// fn main() {
///     let reg_ptr = &mut 0_u32 as *mut u32;
///     let mut reg = Status::Register::new(reg_ptr);
///     reg.modify(Status::Dead::Field::checked::<U1>());
///     assert_eq!(unsafe { *reg_ptr }, 2);
/// }
/// ```
#[macro_export]
macro_rules! register {
    {
        $name:ident,
        $mode:ident,
        Fields [$($fields:tt)*]
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $name {
            use typenum::consts::*;
            use typenum::{Unsigned, IsGreater};
            use $crate::{Field as F, Pointer, Positioned};

            mode!($mode);

            fields!($($fields)*);

        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! fields {
    {
        $(#[$outer:meta])*
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) [ $($enums:tt)* ] $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        pub mod $name {

            use super::*;

            $(#[$outer])*
            pub type Field = F<op!(((U1 << $width) - U1) << $offset), $offset, op!((U1 << $width) - U1)>;

            /// In order to read a field, an instance
            /// of that field must be given to have access to its mask and offset. `Read`
            /// can be used as an argument to `get_field` so one does not have to
            /// construct an arbitrary one when doing a read.
            pub const Read: Field = Field::checked::<U0>();

            /// A field whose value is `$field_max`.
            /// Passing it to `modify` will set that field to its max value in the
            /// register. This is useful particularly in the case of single-bit wide
            /// fields.
            pub const Set: Field = Field::checked::<op!((U1 << $width) - U1)>();

            /// A field whose value is zero. Passing
            /// it to `modify` will clear that field in the register.
            pub const Clear: Field = Field::checked::<U0>();

            /// Constants mapping the enum-like field names to values.
            enums!($($enums)*);
        }

        fields!($($rest)*);
    };
    {
        $(#[$outer:meta])*
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        pub mod $name {

            use super::*;

            $(#[$outer])*
            pub type Field = F<op!(((U1 << $width) - U1) << $offset), $offset, op!((U1 << $width) - U1)>;

            pub const Read: Field = Field::checked::<U0>();
            pub const Set: Field = Field::checked::<op!((U1 << $width) - U1)>();
            pub const Clear: Field = Field::checked::<U0>();
        }

        fields!($($rest)*);
    };
    (, $($rest:tt)*) => (fields!($($rest)*););
    () => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! enums {
    {
        $(

            $(#[$outer:meta])*
            $name:ident = $val:ident
        ),*
    } => {
        $(
            $(#[$outer])*
            pub const $name: Field = Field::checked::<$val>();
        )*
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! mode {
    (RO) => {
        use $crate::ReadOnlyRegister;

        pub struct Register {
            ptr: *const u32,
        }

        impl Register {

            /// `new` constructs a read-only register around the given
            /// pointer.
            pub fn new(ptr: *const u32) -> Self {
                Self { ptr }
            }

        }

        impl $crate::ReadOnlyRegister for Register {
            /// `get_field` takes a field and sets the value of that
            /// field to its value in the register.
            fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
                &self,
                f: F<M, O, U>,
            ) -> Option<F<M, O, U>>
            where
                U: IsGreater<U0, Output = B1>,
            {
                f.set(unsafe { (*self.ptr & M::U32) >> O::U32 })
            }

            /// `read` returns the current state of the register as a `u32`.
            fn read(&self) -> u32 {
                unsafe { *self.ptr }
            }
        }
    };
    (WO) => {

        use $crate::WriteOnlyRegister;

        pub struct Register {
            ptr: *mut u32,
        }

        impl Register {
            /// `new` constructs a write-only register around the
            /// given pointer.
            pub fn new(ptr: mut u32) -> Self {
                Self { ptr }
            }

        }

        impl $crate::WriteOnlyRegister for Register {
            /// `modify` takes one or more fields, joined by `+`, and
            /// sets those fields in the register, leaving the others
            /// as they were.
            fn modify<V: Positioned>(&mut self, val: V) {
                unsafe { *self.ptr = *self.ptr | val.in_position() };
            }

            /// `write` sets the value of the whole register to the
            /// given `u32` value.
            fn write(&mut self, val: u32) {
                unsafe { *self.ptr = val };
            }
        }

    };
    (RW) => {

        pub struct Register {
            ptr: *mut u32,
        }

        impl Register {
            /// `new` constructs a read-write register around the
            /// given pointer.
            pub fn new(ptr: *mut u32) -> Self {
                Self { ptr }
            }

        }

        impl $crate::ReadWriteRegister for Register {
            /// `get_field` takes a field and sets the value of that
            /// field to its value in the register.
            fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
                &self,
                f: F<M, O, U>,
            ) -> Option<F<M, O, U>>
            where
                U: IsGreater<U0, Output = B1>,
            {
                f.set(unsafe { (*self.ptr & M::U32) >> O::U32 })
            }

            /// `read` returns the current state of the register as a `u32`.
            fn read(&self) -> u32 {
                unsafe { *self.ptr }
            }

            /// `modify` takes one or more fields, joined by `+`, and
            /// sets those fields in the register, leaving the others
            /// as they were.
            fn modify<V: Positioned>(&mut self, val: V) {
                unsafe { *self.ptr = *self.ptr | val.in_position() };
            }

            /// `write` sets the value of the whole register to the
            /// given `u32` value.
            fn write(&mut self, val: u32) {
                unsafe { *self.ptr = val };
            }
        }
    };
}

#[cfg(test)]
mod test {
    use typenum::consts::U1;

    use crate::{ReadOnlyRegister, ReadWriteRegister};

    register! {
        Status,
        RW,
        Fields [
            /// Here I'm just testing that doc comments work.
            On WIDTH(U1) OFFSET(U0),
            Color WIDTH(U3) OFFSET(U2) [
                /// In here too!
                // Even with a bunch of lines.
                Red = U1,
                Blue = U2,
                Green = U3,
                Yellow = U4
            ],
            Dead WIDTH(U1) OFFSET(U1)
        ]
    }

    #[test]
    fn test_rw_macro() {
        let reg_ptr = &mut 0_u32 as *mut u32;
        let mut reg = Status::Register::new(reg_ptr);
        reg.modify(Status::Dead::Field::checked::<U1>());
        assert_eq!(unsafe { *reg_ptr }, 2);
    }

    register! {
        RNG,
        RO,
        Fields [
            /// This field means the RNG is working on generating a
            /// random number.
            Working WIDTH(U1) OFFSET(U0),
            Width WIDTH(U2) OFFSET(U1) [
                Four = U0,
                Eight = U1,
                Sixteen = U2
            ]
        ]
    }

    #[test]
    fn test_ro_macro() {
        let ptr = &mut 4_u32 as *mut u32;
        let reg = RNG::Register::new(ptr);
        let width = reg.get_field(RNG::Width::Read).unwrap();
        assert_eq!(width, RNG::Width::Sixteen);
    }

    #[test]
    fn test_field_disj() {
        let reg_ptr = &mut 0_u32 as *mut u32;
        let mut reg = Status::Register::new(reg_ptr);
        reg.modify(Status::Dead::Set + Status::Color::Blue + Status::On::Clear);
        assert_eq!(unsafe { *reg_ptr }, 10);
    }
}
