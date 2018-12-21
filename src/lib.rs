#![no_std]
#![feature(const_fn)]

extern crate type_bounds;
extern crate typenum;

use core::marker::PhantomData;
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
    mask: PhantomData<M>,
    offset: PhantomData<O>,
    val: BoundedU32<V, L, U>,
}

impl<M: Unsigned, O: Unsigned, V: Unsigned, L: Unsigned, U: Unsigned>
    Field<M, O, V, L, U>
where
    V: IsLessOrEqual<U, Output = B1>,
    V: IsGreaterOrEqual<L, Output = B1>,
{
    pub const fn new() -> Self {
        Field {
            mask: PhantomData,
            offset: PhantomData,
            val: BoundedU32::new(V::U32),
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

pub struct Register<N: Unsigned + UnsignedLike>(BoundedU32<N, U0, U255>)
where
    N: IsLessOrEqual<U255, Output = B1>,
    N: IsGreaterOrEqual<U0, Output = B1>;

impl<N: Unsigned + UnsignedLike> Register<N>
where
    N: IsLessOrEqual<U255, Output = B1>,
    N: IsGreaterOrEqual<U0, Output = B1>,
{
    /// The math to modify a field is as follows:
    /// ```not_rust
    /// (register.value & !field.mask) | (field.value << field.offset)
    /// ```
    pub fn modify<
        M: Unsigned + UnsignedLike,
        O: Unsigned + UnsignedLike,
        V: Unsigned + UnsignedLike,
        L: Unsigned,
        U: Unsigned,
    >(
        _f: Field<M, O, V, L, U>,
    ) -> Register<
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output,
    >
    where
        V: IsLessOrEqual<U, Output = B1>,
        V: IsGreaterOrEqual<L, Output = B1>,
        N: BitAnd<<M as Not>::Output>,
        V: Shl<O>,

        <N as BitAnd<<M as Not>::Output>>::Output: BitOr<<V as Shl<O>>::Output>,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: Unsigned,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: UnsignedLike,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: IsLessOrEqual<U255, Output = B1>,
        <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
            <V as Shl<O>>::Output,
        >>::Output: IsGreaterOrEqual<U0, Output = B1>,
    {
        Register(BoundedU32::new(
            <<N as BitAnd<<M as Not>::Output>>::Output as BitOr<
                <V as Shl<O>>::Output,
            >>::Output::U32,
        ))
    }

    /// The math to read a field is as follows:
    /// ```not_rust
    /// (register.value & field.mask) >> field.offset
    /// ```
    pub fn read<
        M: Unsigned + UnsignedLike,
        O: Unsigned + UnsignedLike,
        V: Unsigned + UnsignedLike,
        L: Unsigned,
        U: Unsigned,
    >(
        &self,
        _f: Field<M, O, V, L, U>,
    ) -> u32
    where
        V: IsLessOrEqual<U, Output = B1>,
        V: IsGreaterOrEqual<L, Output = B1>,
        N: BitAnd<M>,
        O: Shr<<N as BitAnd<M>>::Output>,

        <O as Shr<<N as BitAnd<M>>::Output>>::Output: Unsigned,
        <O as Shr<<N as BitAnd<M>>::Output>>::Output: UnsignedLike,
        <O as Shr<<N as BitAnd<M>>::Output>>::Output:
            IsLessOrEqual<U, Output = B1>,
        <O as Shr<<N as BitAnd<M>>::Output>>::Output:
            IsGreaterOrEqual<L, Output = B1>,
    {
        <O as Shr<<N as BitAnd<M>>::Output>>::Output::U32
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
