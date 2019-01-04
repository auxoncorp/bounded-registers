use core::marker::PhantomData;
use core::ops::{Add, BitAnd, BitOr, Not, Shl, Shr};

use typenum::consts::B1;
use typenum::{IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use type_bounds::num::BoundedU32;

/// A RWField represents a field within a register. It's type params are
/// defined as follows:
//
// - `M` :: This the type level representation of the `RWField`'s mask.
// - `O` :: This the type level representation of the `RWField`'s offset.
// - `V` :: This the type level representation of the `RWField`'s current value.
// - `L` & `U` :: These represent the range in which `V` must fall.
pub struct RWField<
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
    RWField<M, O, V, L, U>
where
    V: IsLessOrEqual<U, Output = B1>,
    V: IsGreaterOrEqual<L, Output = B1>,
{
    /// set produces a `RWField` whose
    /// - mask is `M`
    /// - offset is `O`
    /// - value is `V`
    /// - lower bound is `L`
    /// - upper bound is `U`
    pub const fn set() -> Self {
        RWField {
            _mask: PhantomData,
            _offset: PhantomData,
            _val: BoundedU32::new(),
        }
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
trait With<Rhs> {
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
    > With<RWField<MR, OR, VR, LR, UR>> for RWField<ML, OL, VL, LL, UL>
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
    > With<RWField<MR, OR, VR, LR, UR>> for V
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
    > With<V> for RWField<ML, OL, VL, LL, UL>
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
/// system. It contains `RWField`s, the logic to extract those fields,
/// and the ability to update the values in those `RWField`s.
///
/// Its bounds represent the total size of the register.
pub struct RWRegister<N: Unsigned, L: Unsigned, U: Unsigned>(
    BoundedU32<N, L, U>,
)
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>;

impl<N: Unsigned, L: Unsigned, U: Unsigned> RWRegister<N, L, U>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    /// new returns a new register whose value is `N`.
    pub fn new() -> Self {
        Self(BoundedU32::new())
    }

    pub fn val(&self) -> u32 {
        self.0.val()
    }

    /// The math to modify a field is as follows:
    /// ```not_rust
    /// (register.value & !field.mask) | (field.value << field.offset)
    /// ```
    pub fn modify_field<
        M: Unsigned,
        O: Unsigned,
        V: Unsigned,
        FL: Unsigned,
        FU: Unsigned,
    >(
        self,
        _f: RWField<M, O, V, FL, FU>,
    ) -> RWRegister<
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output,
        L,
        U,
    >
    where
        V: IsLessOrEqual<FU, Output = B1>,
        V: IsGreaterOrEqual<FL, Output = B1>,
        V: Shl<O>,
        M: Not,
        N: BitAnd<<M as Not>::Output>,

        <N as BitAnd<<M as Not>::Output>>::Output: BitOr<<V as Shl<O>>::Output>,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: Unsigned,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: IsLessOrEqual<U, Output = B1>,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: IsGreaterOrEqual<L, Output = B1>,
    {
        RWRegister(BoundedU32::new())
    }

    pub fn modify<V: Unsigned>(&self) -> RWRegister<V, L, U>
    where
        V: IsLessOrEqual<U, Output = B1>,
        V: IsGreaterOrEqual<L, Output = B1>,
    {
        RWRegister(BoundedU32::new())
    }

    /// The math to read a field is as follows:
    /// ```not_rust
    /// (register.value & field.mask) >> field.offset
    /// ```
    pub fn read<
        M: Unsigned,
        O: Unsigned,
        V: Unsigned,
        FL: Unsigned,
        FU: Unsigned,
    >(
        &self,
        _f: RWField<M, O, V, FL, FU>,
    ) -> u32
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
        <<N as BitAnd<M>>::Output as Shr<O>>::Output::U32
    }
}

#[cfg(test)]
mod test {

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

        use typenum::consts::{U0, U1, U2, U28, U3, U4, U7};

        pub type On<N> = RWField<U1, U0, N, U0, U1>;
        pub type Dead<N> = RWField<U2, U1, N, U0, U1>;
        pub type Color<N> = RWField<U28, U2, N, U0, U7>;

        #[allow(unused)]
        #[allow(non_upper_case_globals)]
        pub mod ColorValues {
            use super::*;

            pub const Red: Color<U1> = Color::set();
            pub const Blue: Color<U2> = Color::set();
            pub const Green: Color<U3> = Color::set();
            pub const Yellow: Color<U4> = Color::set();
        }
    }

    use super::*;

    use typenum::consts::{U0, U255};

    type EightBitRWRegister<N> = RWRegister<N, U0, U255>;

    #[test]
    fn test_reg() {
        let reg: EightBitRWRegister<U0> = RWRegister(BoundedU32::zero());
        let reg_prime = reg.modify_field(Status::ColorValues::Blue);

        assert_eq!(reg_prime.val(), 8_u32);
        assert_eq!(reg_prime.read(Status::Color::<U0>::set()), 2_u32);
    }
}
