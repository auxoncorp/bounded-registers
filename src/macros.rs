#[macro_export]
macro_rules! register {
    {
        $name:ident,
        $width:ident,
        $mode:ident,
        Fields [$($fields:tt)*]
    } => {
        #[allow(unused)]
        #[allow(non_snake_case)]
        pub mod $name {
            use typenum::consts::*;
            use typenum::{Unsigned, IsGreater};
            use $crate::register::{Field as F, Pointer, Positioned};

            mode!($mode);

            fields!($($fields)*);

        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! fields {
    {
        $name:ident WIDTH($width:ident) OFFSET($offset:ident) [ $($enums:tt)* ] $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        pub mod $name {

            use super::*;

            pub type Field = F<op!(((U1 << $width) - U1) << $offset), $offset, op!((U1 << $width) - U1)>;

            pub const Read: Field = Field::checked::<U0>();
            pub const Set: Field = Field::checked::<op!((U1 << $width) - U1)>();
            pub const Clear: Field = Field::checked::<U0>();

            enums!($($enums)*);
        }

        fields!($($rest)*);
    };
    {
        $name:ident WIDTH($width:ident) OFFSET($offset:ident), $($rest:tt)*
    } => {
        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        pub mod $name {

            use super::*;

            pub type Field = F<op!(((U1 << $width) - U1) << $offset), $offset, op!((U1 << $width) - U1)>;

            pub const Read: Field = Field::checked::<U0>();
            pub const Set: Field = Field::checked::<op!((U1 << $width) - U1)>();
            pub const Clear: Field = Field::checked::<U0>();
        }

        fields!($($rest)*);
    };
    () => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! enums {
    {
        $( $name:ident = $val:ident),*
    } => {
        $(
            pub const $name: Field = Field::checked::<$val>();
        )*
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! mode {
    (RO) => {
        pub struct Register {
            ptr: *const u32,
        }

        impl Register {
            pub fn new(ptr: *const u32) -> Self {
                Self { ptr }
            }

            pub fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
                &self,
                f: F<M, O, U>,
            ) -> Option<F<M, O, U>>
            where
                U: IsGreater<U0, Output = B1>,
            {
                f.set(unsafe { (*self.ptr & M::U32) >> O::U32 })
            }

            pub fn read(&self) -> u32 {
                unsafe { *self.ptr }
            }
        }
    };
    (WO) => {
        pub struct Register {
            ptr: *mut u32,
        }

        impl Register {
            pub fn new(ptr: mut u32) -> Self {
                Self { ptr }
            }

            pub fn write_field<M: Unsigned, O: Unsigned, U: Unsigned>(
                &mut self,
                f: Field<M, O, U>,
            ) where
                U: IsGreater<U0, Output = B1>,
            {
                unsafe { *self.ptr = (*self.ptr & !M::U32) | (f.val() << O::U32) }
            }

            pub fn modify<V: Positioned>(&mut self, val: V) {
                unsafe { *self.ptr = *self.ptr | val.in_position() };
            }

            pub fn write(&mut self, val: u32) {
                unsafe { *self.ptr = val };
            }
        }

    };
    (RW) => {
        pub struct Register {
            ptr: *mut u32,
        }

        impl Register {
            pub fn new(ptr: *mut u32) -> Self {
                Self { ptr }
            }

            pub fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
                &self,
                f: F<M, O, U>,
            ) -> Option<F<M, O, U>>
            where
                U: IsGreater<U0, Output = B1>,
            {
                f.set(unsafe { (*self.ptr & M::U32) >> O::U32 })
            }

            pub fn read(&self) -> u32 {
                unsafe { *self.ptr }
            }

            pub fn write_field<M: Unsigned, O: Unsigned, U: Unsigned>(
                &mut self,
                f: F<M, O, U>,
            ) where
                U: IsGreater<U0, Output = B1>,
            {
                unsafe { *self.ptr = (*self.ptr & !M::U32) | (f.val() << O::U32) }
            }

            pub fn modify<V: Positioned>(&mut self, val: V) {
                unsafe { *self.ptr = *self.ptr | val.in_position() };
            }

            pub fn write(&mut self, val: u32) {
                unsafe { *self.ptr = val };
            }
        }
    };
}

#[cfg(test)]
mod test {
    use typenum::consts::U1;

    register! {
        Status,
        u8,
        RW,
        Fields [
            On WIDTH(U1) OFFSET(U0),
            Dead WIDTH(U1) OFFSET(U1),
            Color WIDTH(U3) OFFSET(U2) [
                Red = U1,
                Blue = U2,
                Green = U3,
                Yellow = U4
            ]
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
        u32,
        RO,
        Fields [
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
