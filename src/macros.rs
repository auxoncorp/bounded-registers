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
/// 2. The permissions for this register. There are two options:
///    * read-write (`RW`)
///    * read-only, (`RO`)
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
/// ### Nota Bené
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
            use typenum::consts::*;
            use $crate::read_write::Register as R;


            pub type Register<N> = R<N, U0, reg_ubounds!($width)>;

            rw_fields!($($rest)*);
        }
        reg_mode!($mode);
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
    (RW) => {
        use $crate::read_write::Read;
        use $crate::read_write::Write;
    };
    (WO) => {
        use $crate::read_write::Write;
    };
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

#[cfg(test)]
mod test {

    use typenum::consts::{U0, U1};

    register! {
        Status,
        u8,
        RW,
        On WIDTH(U1) OFFSET(U0),
        Dead WIDTH(U1) OFFSET(U1),
        Color WIDTH(U3) OFFSET(U2) [
            Red = U1,
            Blue = U2,
            Green = U3,
            Yellow = U4
        ]
    }

    #[test]
    fn test_reg_macro_rw() {
        let val = &mut 0_u32 as *mut u32;
        let reg = Status::Register::<U0>::new(val);
        let reg_prime = reg.modify_field(Status::Color::Blue::Term);
        assert_eq!(reg_prime.val(), 8_u32);
        assert_eq!(
            reg_prime.read(Status::Color::Read),
            Status::Color::Blue::Term
        );
    }

    register! {
        RNG,
        u32,
        RO,
        Working WIDTH(U1) OFFSET(U0),
        Width WIDTH(U2) OFFSET(U1) [
            Four = U0,
            Eight = U1,
            Sixteen = U2
        ]
    }

    #[test]
    fn test_reg_macro_ro() {
        let ptr = &4_u32 as *const u32;
        let reg = RNG::Register::new(ptr);
        let width = reg.read(RNG::Width::Read).unwrap();
        assert_eq!(width.val(), RNG::Width::Values::Sixteen);
    }

    #[test]
    fn test_with() {
        use super::super::read_write::With;
        type On = Status::On::Field<U1>;
        type Dead = Status::Dead::Field<U1>;
        type Blue = Status::Color::Blue::Type;

        let val = &mut 0_u32 as *mut u32;
        let reg: Status::Register<U0> = Status::Register::new(val);
        let reg_prime: Status::Register<with!(On + Dead + Blue)> = reg.modify();
        assert_eq!(reg_prime.val(), 11_u32);
        assert_eq!(
            reg_prime.read(Status::Color::Read),
            Status::Color::Blue::Term
        );
    }
}
