#![no_std]
#![feature(const_fn)]

extern crate type_bounds;
extern crate typenum;

use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

use typenum::consts::{B1, U0, U255};
use typenum::{IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use type_bounds::num::BoundedU32;

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
    mask: M,
    offset: O,
    val: BoundedU32<V, L, U>,
}

impl<M: Unsigned, O: Unsigned, V: Unsigned, L: Unsigned, U: Unsigned>
    Field<M, O, V, L, U>
where
    V: IsLessOrEqual<U, Output = B1>,
    V: IsGreaterOrEqual<L, Output = B1>,
{
    pub const fn new(mask: M, offset: O) -> Self {
        Field {
            mask: mask,
            offset: offset,
            val: BoundedU32::zero(),
        }
    }

    pub fn modify<N: Unsigned>(
        self,
        val: BoundedU32<N, L, U>,
    ) -> Field<M, O, N, L, U>
    where
        N: IsLessOrEqual<U, Output = B1>,
        N: IsGreaterOrEqual<L, Output = B1>,
    {
        Field {
            mask: self.mask,
            offset: self.offset,
            val: val,
        }
    }
}

pub trait UnsignedLike: Copy + Eq + Not + BitAnd + BitOr + Shl + Shr {}

impl<N: Unsigned, L: Unsigned, U: Unsigned> UnsignedLike for BoundedU32<N, L, U>
where
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,

    N: Not,
    <N as Not>::Output: Unsigned,
    <N as Not>::Output: IsLessOrEqual<U, Output = B1>,
    <N as Not>::Output: IsGreaterOrEqual<L, Output = B1>,

    N: BitAnd,
    <N as BitAnd>::Output: Unsigned,
    <N as BitAnd>::Output: IsLessOrEqual<U, Output = B1>,
    <N as BitAnd>::Output: IsGreaterOrEqual<L, Output = B1>,

    N: BitOr,
    <N as BitOr>::Output: Unsigned,
    <N as BitOr>::Output: IsLessOrEqual<U, Output = B1>,
    <N as BitOr>::Output: IsGreaterOrEqual<L, Output = B1>,

    N: Shl,
    <N as Shl>::Output: Unsigned,
    <N as Shl>::Output: IsLessOrEqual<U, Output = B1>,
    <N as Shl>::Output: IsGreaterOrEqual<L, Output = B1>,

    N: Shr,
    <N as Shr>::Output: Unsigned,
    <N as Shr>::Output: IsLessOrEqual<U, Output = B1>,
    <N as Shr>::Output: IsGreaterOrEqual<L, Output = B1>,
{
}

pub struct Register<N: Unsigned>(BoundedU32<N, U0, U255>)
where
    N: IsLessOrEqual<U255, Output = B1>,
    N: IsGreaterOrEqual<U0, Output = B1>;

impl<N: Unsigned + UnsignedLike> Register<N>
where
    N: IsLessOrEqual<U255, Output = B1>,
    N: IsGreaterOrEqual<U0, Output = B1>,
{
    pub fn clear(self) -> BoundedU32<U0, U0, U255> {
        BoundedU32::zero()
    }

    pub fn set(self) -> BoundedU32<U255, U0, U255> {
        BoundedU32::new(255)
    }

    fn modify<
        M: Unsigned + UnsignedLike,
        O: Unsigned + UnsignedLike,
        V: Unsigned + UnsignedLike,
        L: Unsigned,
        U: Unsigned,
    >(
        self,
        field: Field<M, O, V, L, U>,
    ) -> Self
    where
        V: IsLessOrEqual<U, Output = B1>,
        V: IsGreaterOrEqual<L, Output = B1>,
    {
        (self.0 & !field.mask) | (field.val << field.offset)
    }
}

#[cfg(test)]
mod test {

    // Going to define the following register:
    // ```
    // decl_register! {
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
}
