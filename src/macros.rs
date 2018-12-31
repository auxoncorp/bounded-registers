#[macro_export]
macro_rules! register {
    {
        $reg_name:ident,
        $reg_width:ident,
        $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $reg_name {
            use $crate::{Field as F, Register as Reg};
            use typenum::consts::*;

            decl_register!($reg_width);

            decl_fields!($($rest)*);
        }
    }
}

macro_rules! decl_register {
    (u8) => {
        pub type Register<N> = Reg<N, U0, op!(U255)>;
    };
    (u16) => {
        pub type Register<N> = Reg<N, U0, op!(U65536 - U1)>;
    };
    (u32) => {
        pub type Register<N> = Reg<N, U0, op!(U4294967296 - U1)>;
    };
}

macro_rules! decl_fields {
    {
        $field_name:ident MAX($field_max:ident) OFFSET($field_offset:ident) [ $($enum_name:ident = $enum_val:ident),* ]
        $($rest:tt)*
    } => {
        pub mod $field_name {
            use super::*;

            pub type Field<N> = F<op!($field_max << $field_offset), $field_offset, N, U0, $field_max>;

            #[allow(unused)]
            #[allow(non_upper_case_globals)]
            pub mod Values {
                use super::*;

                $(
                    pub const $enum_name: Field<$enum_val> = Field::set();
                )*
            }
        }
        decl_fields!($($rest)*);
    };
    {
        $field_name:ident MAX($field_max:ident) OFFSET($field_offset:ident)
        $($rest:tt)*
    } => {
        pub mod $field_name {
            use super::*;

            pub type Field<N> = F<op!($field_max << $field_offset), $field_offset, N, U0, $field_max>;
        }
        decl_fields!($($rest)*);
    };
    () => ();

    // This is a catch-all: if we have some unmatched rule that's just a comma,
    // toss it and keep recursing.
    (, $($rest:tt)*) => (decl_fields!($($rest)*);)
}

#[cfg(test)]
mod test {

    use typenum::consts::U0;

    register! {
        Status,
        u8,
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
    fn test_reg_macro() {
        let reg = Status::Register::<U0>::new();
        let reg_prime = reg.modify(Status::Color::Values::Blue);
        assert_eq!(reg_prime.val(), 8_u32);
    }
}
