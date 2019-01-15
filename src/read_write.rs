use core::marker::PhantomData;
use core::ops::{BitOr, Shl};

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
