use core::marker::PhantomData;
use core::ops::Add;

use typenum::consts::{B1, U0};
use typenum::{IsGreater, IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use type_bounds::num::runtime::Bounded;

/// A field in a register parameterized by its mask, offset, and upper
/// bound.
#[derive(Debug)]
pub struct Field<M: Unsigned, O: Unsigned, U: Unsigned>
where
    U: IsGreater<U0, Output = B1>,
{
    val: Bounded<u32, U0, U>,
    _mask: PhantomData<M>,
    _offset: PhantomData<O>,
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    pub fn new(val: u32) -> Option<Self> {
        Bounded::new(val).map(|val| Self {
            val: val,
            _offset: PhantomData,
            _mask: PhantomData,
        })
    }

    pub fn set(mut self, val: u32) -> Option<Self> {
        Bounded::new(val).map(|val| {
            self.val = val;
            self
        })
    }

    pub const fn checked<V: Unsigned>() -> Self
    where
        V: IsLessOrEqual<U, Output = B1>,
        V: IsGreaterOrEqual<U0, Output = B1>,
    {
        Self {
            val: Bounded::checked::<V>(),
            _offset: PhantomData,
            _mask: PhantomData,
        }
    }

    pub fn val(&self) -> u32 {
        self.val.val
    }
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> PartialEq<Field<M, O, U>>
    for Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    fn eq(&self, rhs: &Field<M, O, U>) -> bool {
        self.val() == rhs.val()
    }
}

pub trait Positioned {
    fn in_position(&self) -> u32;
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Positioned for Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    fn in_position(&self) -> u32 {
        self.val() << O::U32
    }
}

pub struct FieldDisj(u32);

impl Positioned for FieldDisj {
    fn in_position(&self) -> u32 {
        self.0
    }
}

impl<
        LM: Unsigned,
        LO: Unsigned,
        LU: Unsigned,
        RM: Unsigned,
        RO: Unsigned,
        RU: Unsigned,
    > Add<Field<RM, RO, RU>> for Field<LM, LO, LU>
where
    LU: IsGreater<U0, Output = B1>,
    RU: IsGreater<U0, Output = B1>,
{
    type Output = FieldDisj;

    fn add(self, rhs: Field<RM, RO, RU>) -> Self::Output {
        FieldDisj((self.val() << LO::U32) | (rhs.val() << RO::U32))
    }
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Add<FieldDisj> for Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    type Output = FieldDisj;

    fn add(self, rhs: FieldDisj) -> Self::Output {
        FieldDisj((self.val() << O::U32) | rhs.0)
    }
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Add<Field<M, O, U>> for FieldDisj
where
    U: IsGreater<U0, Output = B1>,
{
    type Output = FieldDisj;

    fn add(self, rhs: Field<M, O, U>) -> Self::Output {
        FieldDisj(self.0 | (rhs.val() << O::U32))
    }
}

pub trait Pointer {
    unsafe fn ptr(&self) -> *mut u32;
}
