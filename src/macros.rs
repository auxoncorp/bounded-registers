/// The `register!` macro generates the necessary parts for managing a register.
/// Use it like so:
/// ```
/// #[macro_use]
/// extern crate registers;
/// #[macro_use]
/// extern crate typenum;
///
/// register! {
///     Status,
///     u8,
///     RW,
///     On WIDTH(U1) OFFSET(U0),
///     Dead WIDTH(U1) OFFSET(U1),
///     Color WIDTH(U3) OFFSET(U2) [
///         Red = U1,
///         Blue = U2,
///         Green = U3,
///         Yellow = U4
///     ]
/// }
/// # fn main() {}
/// ```
/// The fields are, in order:
/// 1. The register's name.
/// 2. The permissions for this register. There are three options:
///    * read-write (`RW`)
///    * read-only, (`RO`)
///    * write-only, (`WO`)
///    Any other entry here will cause an expansion failure.
/// 3. A field list. A field declaration requires that fields offset and its
///    maximum value. See the section below on how to use fields.
///
/// ### Fields
/// A field may be declared in two ways; either with or
/// without enumerated values. The example above shows both. `On` and
/// `Dead` are declared only with their width and offset, while
/// `Color` also declares its possible values with their names and/or
/// meanings.
///
/// ### NB:
/// Notice that width, offset, and the enumerated values are stated in terms of
/// their type-level numbers, prepended with `U`. This is because `registers`
/// uses type-level math in its generated code for as many operations as
/// possible for safer and faster register access.
#[macro_export]
macro_rules! register {
    {
        $name:ident,
        $width:ident,
        RO,
        $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $name {
            use typenum::consts::*;
            use $crate::read_only::Register as R;

            #[allow(non_camel_case_types)]
            type reg_width = $width;

            pub type Register = R<reg_width>;

            ro_fields!($($rest)*);
        }
    };
    {
        $name:ident,
        $width:ident,
        $mode:ident,
        $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $name {
            use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

            use typenum::{Unsigned, IsLessOrEqual, IsGreaterOrEqual};
            use typenum::consts::*;

            use type_bounds::num::BoundedU32;

            use $crate::read_write::Field;

            /// The logical representation of a register on a physical
            /// system. It contains `Field`s, the logic to extract
            /// those fields, and the ability to update the values in
            /// those `Field`s.
            ///
            /// Its bounds represent the total size of the register.
            pub struct Register<N: Unsigned>
            where
                N: IsLessOrEqual<reg_ubounds!($width), Output = B1>,
                N: IsGreaterOrEqual<U0, Output = B1>,
            {
                ptr: *mut u32,
                val: BoundedU32<N, U0, reg_ubounds!($width)>,
            }

            impl<N: Unsigned> Register<N>
            where
                N: IsLessOrEqual<reg_ubounds!($width), Output = B1>,
                N: IsGreaterOrEqual<U0, Output = B1>,
            {
                pub fn new(ptr: *mut u32) -> Self {
                    unsafe {*ptr = N::U32};
                    Self{
                        ptr:ptr,
                        val: BoundedU32::new(),
                    }
                }

                /// modify takes a `V`, as a new value for the register and sets
                /// the register to that value.
                ///
                /// *NB*: This lives here, on `Register`, because it cannot be
                /// implemented generically without higher-kinded types.
                pub fn modify<V: Unsigned>(self) -> Register<V>
                where
                    V: IsLessOrEqual<reg_ubounds!($width), Output = B1>,
                    V: IsGreaterOrEqual<U0, Output = B1>,
                {
                    unsafe { *self.ptr = V::U32 };
                    Register{
                        val: BoundedU32::new(),
                        ptr: unsafe { self.ptr },
                    }
                }

                /// The math to modify a field is as follows:
                /// ```not_rust
                /// (register.value & !field.mask) | (field.value << field.offset)
                /// ```
                ///
                /// *NB*: This lives here, on `Register`, because it cannot be
                /// implemented generically without higher-kinded types.
                pub fn modify_field<
                    M: Unsigned,
                    O: Unsigned,
                    V: Unsigned,
                    FL: Unsigned,
                    FU: Unsigned,
                >(
                    self,
                    _f: Field<M, O, V, FL, FU>,
                ) -> Register<
                    <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
                    <V as Shl<O>>::Output,
                >>::Output>
                where
                    V: IsLessOrEqual<FU, Output = B1>,
                    V: IsGreaterOrEqual<FL, Output = B1>,
                    V: Shl<O>,
                    M: Not,
                    N: BitAnd<<M as Not>::Output>,

                    <N as BitAnd<<M as Not>::Output>>::Output:
                        BitOr<<V as Shl<O>>::Output>,
                    <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
                        <V as Shl<O>>::Output,
                    >>::Output: Unsigned,
                    <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
                        <V as Shl<O>>::Output,
                    >>::Output: IsLessOrEqual<reg_ubounds!($width), Output = B1>,
                    <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
                        <V as Shl<O>>::Output,
                    >>::Output: IsGreaterOrEqual<U0, Output = B1>,
                {
                    unsafe {
                        *self.ptr =
                            <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
                            <V as Shl<O>>::Output,
                        >>::Output::U32
                    };
                    Register{
                        val: BoundedU32::new(),
                        ptr: unsafe { self.ptr },
                    }
                }

                reg_mode!($mode WIDTH($width));

            }

            rw_fields!($($rest)*);
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! reg_ubounds {
    (u8) => {
        U255
    };
    (u16) => {
        op!(U65536 - U1)
    };
    (u32) => {
        op!(U4294967296 - U1)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! rw_fields {
    {
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) [ $($enum_name:ident = $enum_val:ident),* ] $($rest:tt)*
    } => {

        #[allow(unused)]
        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        pub mod $name {
            use typenum::consts::*;
            use $crate::read_write::Field as F;

            pub type Field<N> = F<op!(((U1 << $width) - U1) << $offset), $offset, N, U0, op!((U1 << $width) - U1)>;

            pub const Read: Field<U0> = Field::new();

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            $(
                pub mod $enum_name {
                    use super::*;
                    pub type Type = Field<$enum_val>;
                    pub const Term: Field<$enum_val> = Field::new();
                }
            )*
        }
        rw_fields!($($rest)*);
    };
    {
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        pub mod $name {
            use typenum::consts::*;
            use $crate::read_write::Field as F;

            pub type Field<N> = F<op!(((U1 << $width) - U1) << $offset), $offset, N, U0, op!((U1 << $width) - U1)>;

            pub const Read: Field<U0> = Field::new();
        }
        rw_fields!($($rest)*);
    };
    (, $($rest:tt)*) => (rw_fields!($($rest)*););
    () => ()
}

#[doc(hidden)]
#[macro_export]
macro_rules! ro_fields {
    {
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) [ $($enum_name:ident = $enum_val:ident),* ] $($rest:tt)*
    } => {

        #[allow(unused)]
        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        pub mod $name {
            use typenum::consts::*;
            use $crate::read_only::Field as F;
            use super::reg_width;

            pub type Field = F<reg_width, op!(((U1 << $width) - U1) << $offset), $offset, U0, op!((U1 << $width) - U1)>;

            pub const Read: Field = Field::zero();

            #[allow(non_upper_case_globals)]
            #[allow(unused)]
            pub mod Values {
                use core::marker::PhantomData;
                use typenum::Unsigned;
                use type_bounds::num::runtime::Bounded;
                use super::*;

                $(
                    pub const $enum_name: reg_width = $enum_val::U32 as reg_width;
                )*
            }
        }
        ro_fields!($($rest)*);
    };
    {
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        pub mod $name {
            use typenum::consts::*;
            use $crate::read_only::Field as F;
            use super::reg_width;

            pub type Field = F<reg_width, op!(((U1 << $width) - U1) << $offset), $offset, U0, op!((U1 << $width) - U1)>;

            pub const Read: Field = Field::zero();
        }
        ro_fields!($($rest)*);
    };
    (, $($rest:tt)*) => (ro_fields!($($rest)*););
    () => ()
}

#[doc(hidden)]
#[macro_export]
macro_rules! reg_mode {
    (RW WIDTH($width:ident)) => {
        /// The math to read a field is as follows:
        /// ```not_rust
        /// (register.value & field.mask) >> field.offset
        /// ```
        pub fn read<M: Unsigned, O: Unsigned, V: Unsigned, FL: Unsigned, FU: Unsigned>(
            &self,
            _f: Field<M, O, V, FL, FU>,
        ) -> Field<M, O, <<N as BitAnd<M>>::Output as Shr<O>>::Output, FL, FU>
        where
            V: IsLessOrEqual<FU, Output = B1>,
            V: IsGreaterOrEqual<FL, Output = B1>,
            N: BitAnd<M>,
            <N as BitAnd<M>>::Output: Shr<O>,

            <<N as BitAnd<M>>::Output as Shr<O>>::Output: Unsigned,
            <<N as BitAnd<M>>::Output as Shr<O>>::Output:
                IsLessOrEqual<FU, Output = B1>,
            <<N as BitAnd<M>>::Output as Shr<O>>::Output:
                IsGreaterOrEqual<FL, Output = B1>,
        {
            Field::new()
        }

        pub fn val(&self) -> u32 {
            self.val.val()
        }
    };
    (WO WIDTH($width:ident)) => {};
}

/// `with!` is used to provide a shorthand for atomically updating many fields
/// on a register. Assuming we have the register declared above, we could use
/// `with!` when making modifications:
/// ```
/// #[macro_use]
/// extern crate registers;
/// #[macro_use]
/// extern crate typenum;
/// use registers::read_write::With;
/// use typenum::consts::{U0, U1};
///
/// register! {
///     Status,
///     u8,
///     RW,
///     On WIDTH(U1) OFFSET(U0),
///     Dead WIDTH(U1) OFFSET(U1),
///     Color WIDTH(U3) OFFSET(U2) [
///         Red = U1,
///         Blue = U2,
///         Green = U3,
///         Yellow = U4
///     ]
/// }
///
/// # fn main() {
/// type OnField = Status::On::Field<U1>;
/// type DeadField = Status::Dead::Field<U1>;
/// type Blue = Status::Color::Blue::Type;
/// let val = &mut 0_u32 as *mut u32;
/// let reg = Status::Register::<U0>::new(val);
/// let reg_prime: Status::Register<with!(OnField + DeadField + Blue)> =
///     reg.modify();
/// assert_eq!(reg_prime.val(), 11_u32);
/// # }
/// ```
#[macro_export]
macro_rules! with {
    {
        $lhs:ident + $rhs:ident
    } => {
        <$lhs as With<$rhs>>::Output
    };
    {
        $lhs:ident + $($rest:tt)*
    } => {
        <$lhs as With<with!($($rest)*)>>::Output
    }
}
