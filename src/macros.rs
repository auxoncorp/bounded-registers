#[macro_export]
macro_rules! register {
    {
        $name:ident,
        $width:ident,
        RW,
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
    };
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
            pub mod Values {
                use super::*;

                $(
                    pub const $enum_name: Field<$enum_val> = Field::new();
                )*
            }
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

    use typenum::consts::U0;

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
        ],
    }

    #[test]
    fn test_reg_macro_rw() {
        let val = &mut 0_u32 as *mut u32;
        let reg = Status::Register::<U0>::new(val);
        let reg_prime = reg.modify_field(Status::Color::Values::Blue);
        assert_eq!(reg_prime.val(), 8_u32);
        assert_eq!(
            reg_prime.read(Status::Color::Read),
            Status::Color::Values::Blue
        );
    }

    register! {
        RNG,
        u32,
        RO,
        Working WIDTH(U1) OFFSET(U0),
        Width WIDTH(U3) OFFSET(U1) [
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

    // #[test]
    // fn test_with() {
    //     use super::super::read_write::With;
    //     type OnField = Status::On::RWField<U1>;
    //     type DeadField = Status::Dead::RWField<U1>;
    //     type Blue = Status::Color::Values::Blue;

    //     let val = &mut 0_u32 as *mut u32;
    //     let reg = Status::RWRegister::<U0>::new(val);
    //     let reg_prime: Status::RWRegister<with!(OnField + DeadField + Blue)>
    // =         reg.modify();
    //     assert_eq!(reg_prime.val(), 11_u32);
    // }
}
