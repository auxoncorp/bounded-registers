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
/// extern crate bounded_registers;
///
/// use typenum::consts::U1;
///
/// register! {
///     Status,
///     u8,
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
///     let mut reg = Status::Register::new(0);
///     reg.modify(Status::Dead::Field::checked::<U1>());
///     assert_eq!(reg.read(), 2);
/// }
/// ```
#[macro_export]
macro_rules! register {
    {
        $(#[$attrs:meta])*
        $name:ident,
        $width:ty,
        $mode:ident,
        Fields [$($fields:tt)*]
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $name {
            use typenum::consts::*;
            use core::marker::PhantomData;
            use typenum::{Unsigned, IsGreater};
            use $crate::{Field as F, Pointer, Positioned};

            use $crate::bounds::{ReifyTo, Reifier};

            use core::ptr;

            type Width = $width;

            #[repr(C)]
            $(#[$attrs])*
            pub struct Register(Width);

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

            type _Offset = $offset;
            type _FieldWidth = $width;

            $(#[$outer])*
            pub type Field = F<super::Width, op!(((U1 << $width) - U1) << $offset), $offset, op!((U1 << $width) - U1), Register>;

            /// In order to read a field, an instance of that field
            /// must be given to have access to its mask and
            /// offset. `Read` can be used as an argument to
            /// `get_field` so one does not have to construct an
            /// arbitrary one when doing a read.
            pub const Read: Field = Field::checked::<U0>();


            /// A field whose value is `$field_max`. Passing it to
            /// `modify` will set that field to its max value in the
            /// register. This is useful particularly in the case of
            /// single-bit wide fields.
            pub const Set: Field = Field::checked::<op!((U1 << $width) - U1)>();

            /// A field whose value is zero. Passing it to `modify`
            /// will clear that field in the register.
            pub const Clear: Field = Read;

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
            pub type Field = F<super::Width, op!(((U1 << $width) - U1) << $offset), $offset, op!((U1 << $width) - U1), Register>;

            /// In order to read a field, an instance of that field
            /// must be given to have access to its mask and
            /// offset. `Read` can be used as an argument to
            /// `get_field` so one does not have to construct an
            /// arbitrary one when doing a read.
            pub const Read: Field = Field::checked::<U0>();

            /// A field whose value is `$field_max`. Passing it to
            /// `modify` will set that field to its max value in the
            /// register. This is useful particularly in the case of
            /// single-bit wide fields.
            pub const Set: Field = Field::checked::<op!((U1 << $width) - U1)>();

            /// A field whose value is zero. Passing it to `modify`
            /// will clear that field in the register.
            pub const Clear: Field = Read;
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
        impl Register {
            /// `new` constructs a read-only register around the given
            /// value.
            pub fn new(init: Width) -> Self {
                Register(init)
            }

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
                U0: ReifyTo<Width>,
            {
                f.set(
                    (unsafe { ptr::read_volatile(&self.0 as *const Width) } & M::reify())
                        >> O::reify(),
                )
            }

            /// `read` returns the current state of the register as a `Width`.
            pub fn read(&self) -> Width {
                unsafe { ptr::read_volatile(&self.0 as *const Width) }
            }

            /// `extract` pulls the state of a register out into a wrapped
            /// read-only register.
            pub fn extract(&self) -> $crate::ReadOnlyCopy<Width, Register> {
                $crate::ReadOnlyCopy(
                    unsafe { ptr::read_volatile(&self.0 as *const Width) },
                    PhantomData,
                )
            }

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
                O: ReifyTo<Width>,
            {
                ((unsafe { ptr::read_volatile(&self.0 as *const Width) } & M::reify())
                    >> O::reify())
                    == U::reify()
            }

            /// `matches_any` returns whether or not any of the given fields
            /// match those fields values inside the register.
            pub fn matches_any<V: Positioned<Width = Width>>(&self, val: V) -> bool {
                (val.in_position() & unsafe { ptr::read_volatile(&self.0 as *const Width) }) != 0
            }

            /// `matches_all` returns whether or not all of the given fields
            /// match those fields values inside the register.
            pub fn matches_all<V: Positioned<Width = Width>>(&self, val: V) -> bool {
                (val.in_position() & unsafe { ptr::read_volatile(&self.0 as *const Width) })
                    == val.in_position()
            }
        }
    };
    (WO) => {
        impl Register {
            /// `new` constructs a write-only register around the
            /// given pointer.
            pub fn new(init: Width) -> Self {
                Register(init)
            }

            /// `modify` takes one or more fields, joined by `+`, and
            /// sets those fields in the register, leaving the others
            /// as they were.
            pub fn modify<V: Positioned<Width = Width>>(&mut self, val: V) {
                unsafe {
                    ptr::write_volatile(
                        &mut self.0 as *mut Width,
                        (ptr::read_volatile(&self.0 as *const Width) & !val.mask())
                            | val.in_position(),
                    );
                };
            }

            /// `write` sets the value of the whole register to the
            /// given `Width` value.
            ///
            /// # Safety
            ///
            /// Bypasses all assurances by writing a raw value directly to the register.
            pub unsafe fn write(&mut self, val: Width) {
                ptr::write_volatile(&mut self.0 as *mut Width, val);
            }
        }
    };
    (RW) => {
        impl Register {
            /// `new` constructs a read-write register around the
            /// given pointer.
            pub fn new(init: Width) -> Self {
                Register(init)
            }

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
                U0: ReifyTo<Width>,
            {
                f.set(
                    (unsafe { ptr::read_volatile(&self.0 as *const Width) } & M::reify())
                        >> O::reify(),
                )
            }

            /// `read` returns the current state of the register as a `Width`.
            pub fn read(&self) -> Width {
                unsafe { ptr::read_volatile(&self.0 as *const Width) }
            }

            /// `extract` pulls the state of a register out into a wrapped
            /// read-only register.
            pub fn extract(&self) -> $crate::ReadOnlyCopy<Width, Register> {
                $crate::ReadOnlyCopy(
                    unsafe { ptr::read_volatile(&self.0 as *const Width) },
                    PhantomData,
                )
            }

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
                O: ReifyTo<Width>,
            {
                ((unsafe { ptr::read_volatile(&self.0 as *const Width) } & M::reify())
                    >> O::reify())
                    == U::reify()
            }

            /// `matches_any` returns whether or not any of the given fields
            /// match those fields values inside the register.
            pub fn matches_any<V: Positioned<Width = Width>>(&self, val: V) -> bool {
                (val.in_position() & unsafe { ptr::read_volatile(&self.0 as *const Width) }) != 0
            }

            /// `matches_all` returns whether or not all of the given fields
            /// match those fields values inside the register.
            pub fn matches_all<V: Positioned<Width = Width>>(&self, val: V) -> bool {
                (val.in_position() & unsafe { ptr::read_volatile(&self.0 as *const Width) })
                    == val.in_position()
            }

            /// `modify` takes one or more fields, joined by `+`, and
            /// sets those fields in the register, leaving the others
            /// as they were.
            pub fn modify<V: Positioned<Width = Width>>(&mut self, val: V) {
                unsafe {
                    ptr::write_volatile(
                        &mut self.0 as *mut Width,
                        (ptr::read_volatile(&self.0 as *const Width) & !val.mask())
                            | val.in_position(),
                    );
                };
            }

            /// `write` sets the value of the whole register to the
            /// given `Width` value.
            ///
            /// # Safety
            ///
            /// Bypasses all assurances by writing a raw value directly to the register.
            pub unsafe fn write(&mut self, val: Width) {
                ptr::write_volatile(&mut self.0 as *mut Width, val);
            }
        }
    };
}

#[cfg(test)]
mod test {
    use typenum::consts::U1;

    register! {
        /// The status register
        #[derive(Debug)]
        Status,
        u8,
        RW,
        Fields [
            /// Here I'm just testing that doc comments work.
            On WIDTH(U1) OFFSET(U0),
            Dead WIDTH(U1) OFFSET(U1),
            Color WIDTH(U3) OFFSET(U2) [
                /// In here too!
                // Even with a bunch of lines.
                Red = U1,
                Blue = U2,
                Green = U3,
                Yellow = U4
            ],
        ]
    }

    #[test]
    fn test_rw_macro() {
        let mut reg = Status::Register::new(0);
        reg.modify(Status::Dead::Field::checked::<U1>());
        assert_eq!(reg.read(), 2);
    }

    #[test]
    fn test_matches_any() {
        let mut reg = Status::Register::new(0);
        reg.modify(Status::Dead::Set);
        assert!(reg.matches_any(Status::On::Set + Status::Dead::Set));
        reg.modify(Status::Dead::Clear);
        assert!(!reg.matches_any(Status::On::Set + Status::Dead::Set));
    }

    #[test]
    fn test_matches_all() {
        let mut reg = Status::Register::new(0);
        reg.modify(Status::Dead::Set + Status::On::Set);
        assert!(reg.matches_all(Status::On::Set + Status::Dead::Set));
        reg.modify(Status::Dead::Clear);
        assert!(!reg.matches_all(Status::On::Set + Status::Dead::Set));
    }

    register! {
        ///  A random number generator
        #[derive(Debug)]
        RNG,
        u8,
        RO,
        Fields [
            /// This field means the RNG is working on generating a
            /// random number.
            Working WIDTH(U1) OFFSET(U0),
            NumWidth WIDTH(U2) OFFSET(U1) [
                Four = U0,
                Eight = U1,
                Sixteen = U2
            ]
        ]
    }

    #[test]
    fn test_ro_macro() {
        let reg = RNG::Register::new(4);
        let width = reg.get_field(RNG::NumWidth::Read).unwrap();
        assert_eq!(width, RNG::NumWidth::Sixteen);
    }

    #[test]
    fn test_field_disj() {
        let mut reg = Status::Register::new(0);
        reg.modify(Status::Dead::Set + Status::Color::Blue + Status::On::Clear);
        assert_eq!(reg.read(), 10);
    }
}
