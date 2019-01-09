use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

use typenum::consts::B1;
use typenum::{IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use type_bounds::num::BoundedU32;

/// A Field represents a field within a register. It's type params are
/// defined as follows:
///
/// - `M` :: This the type level representation of the `Field`'s mask.
/// - `O` :: This the type level representation of the `Field`'s offset.
/// - `V` :: This the type level representation of the `Field`'s current value.
/// - `L` & `U` :: These represent the range in which `V` must fall.
#[derive(Debug)]
pub struct Field<
    M: Unsigned,
    O: Unsigned,
    V: Unsigned,
    L: Unsigned,
    U: Unsigned,
> where
    V: IsLessOrEqual<U, Output = B1>,
    V: IsGreaterOrEqual<L, Output = B1>,
{
    _mask: PhantomData<M>,
    _offset: PhantomData<O>,
    _val: BoundedU32<V, L, U>,
}

impl<M: Unsigned, O: Unsigned, V: Unsigned, L: Unsigned, U: Unsigned>
    Field<M, O, V, L, U>
where
    V: IsLessOrEqual<U, Output = B1>,
    V: IsGreaterOrEqual<L, Output = B1>,
{
    /// new produces a `Field` whose
    /// - mask is `M`
    /// - offset is `O`
    /// - value is `V`
    /// - lower bound is `L`
    /// - upper bound is `U`
    pub const fn new() -> Self {
        Field {
            _mask: PhantomData,
            _offset: PhantomData,
            _val: BoundedU32::new(),
        }
    }

    /// `val` returns the runtime value of the field.
    pub const fn val(&self) -> u32 {
        V::U32
    }
}

impl<M: Unsigned, O: Unsigned, V: Unsigned, L: Unsigned, U: Unsigned>
    PartialEq<Field<M, O, V, L, U>> for Field<M, O, V, L, U>
where
    V: IsLessOrEqual<U, Output = B1>,
    V: IsGreaterOrEqual<L, Output = B1>,
{
    fn eq(&self, _rhs: &Field<M, O, V, L, U>) -> bool {
        true
    }
}

/// With is a trait used to type a collection of fields which all need
/// to be updated atomically on a register. When both sides are
/// fields, its math is as follows:
/// ```not_rust
/// (lhs_field.val << lhs_field.offset) | (rhs_field.val << rhs_field.offset)
/// ```
///
/// Otherwise, when one or both sides are type-level numbers, it's a
/// simple bitwise or operation.
pub trait With<Rhs> {
    type Output;
}

// The implementation for With where both sides are fields.
impl<
        ML: Unsigned,
        OL: Unsigned,
        VL: Unsigned,
        LL: Unsigned,
        UL: Unsigned,
        MR: Unsigned,
        OR: Unsigned,
        VR: Unsigned,
        LR: Unsigned,
        UR: Unsigned,
    > With<Field<MR, OR, VR, LR, UR>> for Field<ML, OL, VL, LL, UL>
where
    VL: IsGreaterOrEqual<LL, Output = B1> + IsLessOrEqual<UL, Output = B1>,
    VR: IsGreaterOrEqual<LR, Output = B1> + IsLessOrEqual<UR, Output = B1>,

    VL: Shl<OL>,
    VR: Shl<OR>,
    <VL as Shl<OL>>::Output: BitOr<<VR as Shl<OR>>::Output>,
{
    type Output =
        <<VL as Shl<OL>>::Output as BitOr<<VR as Shl<OR>>::Output>>::Output;
}

// The implementation of With where the left-hand side is just a
// type-level number.
impl<
        V: Unsigned,
        MR: Unsigned,
        OR: Unsigned,
        VR: Unsigned,
        LR: Unsigned,
        UR: Unsigned,
    > With<Field<MR, OR, VR, LR, UR>> for V
where
    VR: IsGreaterOrEqual<LR, Output = B1> + IsLessOrEqual<UR, Output = B1>,

    VR: Shl<OR>,
    V: BitOr<<VR as Shl<OR>>::Output>,
{
    type Output = <V as BitOr<<VR as Shl<OR>>::Output>>::Output;
}

// The implementation of With where the right-hand side is just a
// type-level number.
impl<
        V: Unsigned,
        ML: Unsigned,
        OL: Unsigned,
        VL: Unsigned,
        LL: Unsigned,
        UL: Unsigned,
    > With<V> for Field<ML, OL, VL, LL, UL>
where
    VL: IsGreaterOrEqual<LL, Output = B1> + IsLessOrEqual<UL, Output = B1>,

    VL: Shl<OL>,
    <VL as Shl<OL>>::Output: BitOr<V>,
{
    type Output = <<VL as Shl<OL>>::Output as BitOr<V>>::Output;
}

// The implementation of With where both sides are type-level numbers.
impl<L: Unsigned, R: Unsigned> With<R> for L
where
    L: BitOr<R>,
{
    type Output = <L as BitOr<R>>::Output;
}

/// The logical representation of a register on a physical
/// system. It contains `Field`s, the logic to extract those fields,
/// and the ability to update the values in those `Field`s.
///
/// Its bounds represent the total size of the register.
pub struct Register<N: Unsigned, L: Unsigned, U: Unsigned>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    ptr: *mut u32,
    val: BoundedU32<N, L, U>,
}

impl<N: Unsigned, L: Unsigned, U: Unsigned> Register<N, L, U>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    /// `new` returns a new register whose value is `N`.
    /// It also sets the register at `ptr` to `N`'s value.
    pub fn new(ptr: *mut u32) -> Self {
        // tie `N` to the value at the pointer given.
        unsafe { *ptr = N::U32 };

        Self {
            val: BoundedU32::new(),
            ptr: ptr,
        }
    }

    pub fn val(&self) -> u32 {
        self.val.val()
    }
}

pub trait Reg {
    unsafe fn ptr(&self) -> *mut u32;
}

impl<N: Unsigned, L: Unsigned, U: Unsigned> Reg for Register<N, L, U>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    unsafe fn ptr(&self) -> *mut u32 {
        self.ptr
    }
}

pub trait Write: Sized + Reg {
    type Lower: Unsigned;
    type Upper: Unsigned;
    type Val: Unsigned;

    /// The math to modify a field is as follows:
    /// ```not_rust
    /// (register.value & !field.mask) | (field.value << field.offset)
    /// ```
    fn modify_field<
        M: Unsigned,
        O: Unsigned,
        V: Unsigned,
        FL: Unsigned,
        FU: Unsigned,
    >(
        self,
        _f: Field<M, O, V, FL, FU>,
    ) -> Register<
        <<Self::Val as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output,
        Self::Lower,
        Self::Upper,
    >
    where
        V: IsLessOrEqual<FU, Output = B1>,
        V: IsGreaterOrEqual<FL, Output = B1>,
        V: Shl<O>,
        M: Not,
        Self::Val: BitAnd<<M as Not>::Output>,

        <Self::Val as BitAnd<<M as Not>::Output>>::Output:
            BitOr<<V as Shl<O>>::Output>,
        <<Self::Val as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: Unsigned,
        <<Self::Val as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: IsLessOrEqual<Self::Upper, Output = B1>,
        <<Self::Val as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: IsGreaterOrEqual<Self::Lower, Output = B1>,
    {
        unsafe {
            *self.ptr() =
                <<Self::Val as BitAnd<<M as Not>::Output>>::Output as BitOr<
                    <V as Shl<O>>::Output,
                >>::Output::U32
        };
        Register {
            val: BoundedU32::new(),
            ptr: unsafe { self.ptr() },
        }
    }

    fn modify<V: Unsigned>(self) -> Register<V, Self::Lower, Self::Upper>
    where
        V: IsLessOrEqual<Self::Upper, Output = B1>,
        V: IsGreaterOrEqual<Self::Lower, Output = B1>,
    {
        unsafe { *self.ptr() = V::U32 };
        Register {
            val: BoundedU32::new(),
            ptr: unsafe { self.ptr() },
        }
    }
}

impl<N: Unsigned, L: Unsigned, U: Unsigned> Write for Register<N, L, U>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    type Val = N;
    type Lower = L;
    type Upper = U;
}

pub trait Read {
    type Val: Unsigned;

    /// The math to read a field is as follows:
    /// ```not_rust
    /// (register.value & field.mask) >> field.offset
    /// ```
    fn read<M: Unsigned, O: Unsigned, V: Unsigned, FL: Unsigned, FU: Unsigned>(
        &self,
        _f: Field<M, O, V, FL, FU>,
    ) -> Field<M, O, <<Self::Val as BitAnd<M>>::Output as Shr<O>>::Output, FL, FU>
    where
        V: IsLessOrEqual<FU, Output = B1>,
        V: IsGreaterOrEqual<FL, Output = B1>,
        Self::Val: BitAnd<M>,
        <Self::Val as BitAnd<M>>::Output: Shr<O>,

        <<Self::Val as BitAnd<M>>::Output as Shr<O>>::Output: Unsigned,
        <<Self::Val as BitAnd<M>>::Output as Shr<O>>::Output:
            IsLessOrEqual<FU, Output = B1>,
        <<Self::Val as BitAnd<M>>::Output as Shr<O>>::Output:
            IsGreaterOrEqual<FL, Output = B1>,
    {
        Field::new()
    }
}

impl<N: Unsigned, L: Unsigned, U: Unsigned> Read for Register<N, L, U>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    type Val = N;
}

#[cfg(test)]
mod test {

    use typenum::consts::{U0, U1, U2, U255, U28, U3, U4, U7};

    // Going to define the following register:
    // ```
    // register! {
    //     STATUS,
    //     u8,
    //     ON WIDTH(1) OFFSET(0),
    //     DEAD WIDTH(1) OFFSET(1),
    //     COLOR WIDTH(3) OFFSET(2) [
    //         RED = 1,
    //         BLUE = 2,
    //         GREEN = 3,
    //         YELLOW = 4,
    //     ]
    // }
    // ```

    #[allow(unused)]
    #[allow(non_snake_case)]
    pub mod Status {
        use super::super::*;
        use super::*;

        pub type On<N> = Field<U1, U0, N, U0, U1>;
        pub type Dead<N> = Field<U2, U1, N, U0, U1>;
        pub type Color<N> = Field<U28, U2, N, U0, U7>;

        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        pub mod ColorValues {
            use super::*;

            pub const Red: Color<U1> = Color::new();
            pub const Blue: Color<U2> = Color::new();
            pub const Green: Color<U3> = Color::new();
            pub const Yellow: Color<U4> = Color::new();
        }
    }

    use super::*;

    type EightBitRegister<N> = Register<N, U0, U255>;

    #[test]
    fn test_reg() {
        let val = &mut 0_u32 as *mut u32;
        let reg: EightBitRegister<U0> = Register::new(val);
        let reg_prime = reg.modify_field(Status::ColorValues::Blue);

        assert_eq!(reg_prime.val(), 8_u32);
        assert_eq!(reg_prime.read(Status::Color::<U0>::new()).val(), 2_u32);
    }

    #[test]
    fn overwrite() {
        // We set the initial value to 1.
        let mut ow_val = 1_u32;
        {
            // Then we give the register a pointer to this value, and
            // set the register's value as 0.
            let ow_val_ptr = &mut ow_val as *mut u32;
            let _reg: EightBitRegister<U0> = Register::new(ow_val_ptr);
        }

        // So when we read the value back out, it should match that of
        // the register's.
        assert_eq!(ow_val, 0);
    }

    #[test]
    fn test_with() {
        let val = &mut 0_u32 as *mut u32;
        let reg: EightBitRegister<U0> = Register::new(val);
        let reg_prime: EightBitRegister<
            <<Field<U1, U0, U1, U0, U1> as With<
                Field<U28, U2, U2, U0, U7>,
            >>::Output as With<Field<U2, U1, U1, U0, U1>>>::Output,
        > = reg.modify();
        assert_eq!(reg_prime.val(), 11);
    }
}
