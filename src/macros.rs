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
///     On MAX(U1) OFFSET(U0),
///     Dead MAX(U1) OFFSET(U1),
///     Color MAX(U7) OFFSET(U2) [
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
///    * write-only (`WO`).
///    Any other entry here will cause an expansion failure.
/// 3. A field list. A field declaration requires that fields offset and its
///    maximum value. See the section below on how to use fields.
///
/// ### Fields
/// A field may be declared in two ways; either with or without enumerated
/// values. The example above shows both. `On` and `Dead` are declared only
/// with their max and offset, while `Color` also declares its possible values
/// with their names and/or meanings.
///
/// ### Nota Bené
/// Notice that max, offset, and the enumerated values are stated in terms of
/// their type-level numbers, prepended with `U`. This is because `registers`
/// uses type-level math in its generated code for as many operations as
/// possible for safer and faster register access.
#[macro_export]
macro_rules! register {
    {
        $reg_name:ident,
        $reg_width:ident,
        RW,
        $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $reg_name {
            use $crate::read_write::{RWField as F, RWRegister as Reg};
            use typenum::consts::*;

            decl_rw_register!($reg_width);

            decl_rw_fields!($($rest)*);
        }
    };
    {
        $reg_name:ident,
        $reg_width:ident,
        RO,
        $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $reg_name {
            use $crate::read_only::{ROField as F, RORegister as Reg};
            use typenum::consts::*;

            decl_ro_register!($reg_width);

            decl_ro_fields!($($rest)*);
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! decl_rw_register {
    (u8) => {
        pub type RWRegister<N> = Reg<N, U0, op!(U255)>;
    };
    (u16) => {
        pub type RWRegister<N> = Reg<N, U0, op!(U65536 - U1)>;
    };
    (u32) => {
        pub type RWRegister<N> = Reg<N, U0, op!(U4294967296 - U1)>;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! decl_ro_register {
    (u8) => {
        pub type RORegister = Reg<u8>;
    };
    (u16) => {
        pub type RORegister = Reg<u16>;
    };
    (u32) => {
        pub type RORegister = Reg<u32>;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! decl_rw_fields {
    {
        $field_name:ident MAX($field_max:ident) OFFSET($field_offset:ident) [ $($enum_name:ident = $enum_val:ident),* ]
        $($rest:tt)*
    } => {
        pub mod $field_name {
            use super::*;

            pub type RWField<N> = F<op!($field_max << $field_offset), $field_offset, N, U0, $field_max>;

            #[allow(unused)]
            #[allow(non_upper_case_globals)]
            pub mod Values {
                use super::*;

                $(
                    pub type $enum_name = RWField<$enum_val>;
                )*
            }
        }
        decl_rw_fields!($($rest)*);
    };
    {
        $field_name:ident MAX($field_max:ident) OFFSET($field_offset:ident)
        $($rest:tt)*
    } => {
        pub mod $field_name {
            use super::*;

            pub type RWField<N> = F<op!($field_max << $field_offset), $field_offset, N, U0, $field_max>;
        }
        decl_rw_fields!($($rest)*);
    };
    // This is a catch-all: if we have some unmatched rule that's just a comma,
    // toss it and keep recursing.
    (, $($rest:tt)*) => (decl_rw_fields!($($rest)*););

    () => ();
}

#[doc(hidden)]
#[macro_export]
macro_rules! decl_ro_fields {
    {
        $field_name:ident MAX($field_max:ident) OFFSET($field_offset:ident) [ $($enum_name:ident = $enum_val:ident),* ]
        $($rest:tt)*
    } => {
        pub mod $field_name {
            use super::*;

            pub type ROField = F<u32, op!($field_max << $field_offset), $field_offset, U0, $field_max>;

            #[allow(unused)]
            #[allow(non_upper_case_globals)]
            pub mod Values {
                use super::*;

                $(
                    pub type $enum_name = ROField;
                )*
            }
        }
        decl_ro_fields!($($rest)*);
    };
    {
        $field_name:ident MAX($field_max:ident) OFFSET($field_offset:ident)
        $($rest:tt)*
    } => {
        pub mod $field_name {
            use super::*;

            pub type RWField = F<u32, op!($field_max << $field_offset), $field_offset, U0, $field_max>;
        }
        decl_ro_fields!($($rest)*);
    };
    // This is a catch-all: if we have some unmatched rule that's just a comma,
    // toss it and keep recursing.
    (, $($rest:tt)*) => (decl_ro_fields!($($rest)*););

    () => ();
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
///     On MAX(U1) OFFSET(U0),
///     Dead MAX(U1) OFFSET(U1),
///     Color MAX(U7) OFFSET(U2) [
///         Red = U1,
///         Blue = U2,
///         Green = U3,
///         Yellow = U4
///     ]
/// }
///
/// # fn main() {
/// type OnField = Status::On::RWField<U1>;
/// type DeadField = Status::Dead::RWField<U1>;
/// type Blue = Status::Color::Values::Blue;
/// let val = &mut 0_u32 as *mut u32;
/// let reg = Status::RWRegister::<U0>::new(val);
/// let reg_prime: Status::RWRegister<with!(OnField + DeadField + Blue)> =
///     reg.modify();
/// assert_eq!(reg_prime.val(), 11_u32);
/// # }
/// ```
#[doc(hidden)]
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
    };
    () => ()
}

#[cfg(test)]
mod test {

    use typenum::consts::{U0, U1};

    register! {
        Status,
        u8,
        RW,
        On MAX(U1) OFFSET(U0),
        Dead MAX(U1) OFFSET(U1),
        Color MAX(U7) OFFSET(U2) [
            Red = U1,
            Blue = U2,
            Green = U3,
            Yellow = U4
        ]
    }

    #[test]
    fn test_reg_macro_rw() {
        let val = &mut 0_u32 as *mut u32;
        let reg = Status::RWRegister::<U0>::new(val);
        let reg_prime = reg.modify_field(Status::Color::Values::Blue::set());
        assert_eq!(reg_prime.val(), 8_u32);
    }

    register! {
        RNG,
        u32,
        RO,
        Working MAX(U1) OFFSET(U0),
        Width MAX(U3) OFFSET(U1) [
            Four = U0,
            Eight = U1,
            Sixteen = U2
        ]
    }

    #[test]
    fn test_reg_macro_ro() {
        let ptr = &4_u32 as *const u32;
        let reg = RNG::RORegister::new(ptr);
        let width = reg.read(RNG::Width::ROField::new(0).unwrap()).unwrap();
        assert_eq!(width.val(), 2);
    }

    #[test]
    fn test_with() {
        use super::super::read_write::With;
        type OnField = Status::On::RWField<U1>;
        type DeadField = Status::Dead::RWField<U1>;
        type Blue = Status::Color::Values::Blue;

        let val = &mut 0_u32 as *mut u32;
        let reg = Status::RWRegister::<U0>::new(val);
        let reg_prime: Status::RWRegister<with!(OnField + DeadField + Blue)> =
            reg.modify();
        assert_eq!(reg_prime.val(), 11_u32);
    }
}
