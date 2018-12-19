#![feature(const_fn)]
#![no_std]
extern crate type_bounds;
extern crate typenum;

use core::ops::{BitAnd, BitOr, Not};

use typenum::{IsGreaterOrEqual, IsLessOrEqual, Unsigned, B1, U0};

use type_bounds::num::BoundedU32;

struct Field<M, O, V> {
    mask: M,
    offset: O,
    val: V,
}

impl<M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned>
    Field<BoundedU32<M, L, U>, BoundedU32<O, L, U>, BoundedU32<U0, L, U>>
where
    U0: IsLessOrEqual<U, Output = B1>,
    U0: IsGreaterOrEqual<L, Output = B1>,
    M: IsLessOrEqual<U, Output = B1>,
    M: IsGreaterOrEqual<L, Output = B1>,
    O: IsLessOrEqual<U, Output = B1>,
    O: IsGreaterOrEqual<L, Output = B1>,
{
    pub const fn new(
        width: BoundedU32<M, L, U>,
        offset: BoundedU32<O, L, U>,
    ) -> Self {
        Field {
            mask: (((1 << (width - 1)) * 2) - 1) << offset,
            offset: offset,
            val: BoundedU32::zero(),
        }
    }
}

impl<N: Unsigned, M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned>
    Field<BoundedU32<M, L, U>, BoundedU32<O, L, U>, BoundedU32<N, L, U>>
where
    U0: IsLessOrEqual<U, Output = B1>,
    U0: IsGreaterOrEqual<L, Output = B1>,
    M: IsLessOrEqual<U, Output = B1>,
    M: IsGreaterOrEqual<L, Output = B1>,
    O: IsLessOrEqual<U, Output = B1>,
    O: IsGreaterOrEqual<L, Output = B1>,
    N: IsLessOrEqual<U, Output = B1>,
    N: IsGreaterOrEqual<L, Output = B1>,
{
    fn set<P: Unsigned, Q: Unsigned>(
        self,
        val: BoundedU32<P, L, U>,
    ) -> Field<BoundedU32<M, L, U>, BoundedU32<O, L, U>, BoundedU32<Q, L, U>>
    where
        P: IsLessOrEqual<U, Output = B1>,
        P: IsGreaterOrEqual<L, Output = B1>,
        Q: IsLessOrEqual<U, Output = B1>,
        Q: IsGreaterOrEqual<L, Output = B1>,

        BoundedU32<M, L, U>: Not,

        BoundedU32<N, L, U>: BitAnd<<BoundedU32<M, L, U> as Not>::Output>,

        <BoundedU32<N, L, U> as BitAnd<<BoundedU32<M, L, U> as Not
            >::Output>>::Output: BitOr<BoundedU32<P, L, U>>,
        Bounded<Q, L, U>: <Bounded<Q, L, U> = BoundedU32<<BoundedU32<N, L, U> as BitAnd<<BoundedU32<M, L, U> as Not>::Output>>::Output as BitOr<BoundedU32<P, L, U>>>::Output>,
    {
        Field {
            mask: self.mask,
            offset: self.offset,
            val: (self.val & !self.mask) | val,
        }
    }
}

#[cfg(test)]
mod test {
    use typenum::consts::{U0, U1, U2, U7};

    use type_bounds::num::BoundedU32;

    use super::*;

    #[test]
    fn test_reg() {
        // Here we're attempting to create the following register:
        // ```
        // decl_register! {
        //     STATUS,
        //     u32,
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

        let _on_field: Field<BoundedU32<U0, U0, U1>, u32> = Field::new(1, 0);
        let _dead_field: Field<BoundedU32<U0, U0, U1>, u32> = Field::new(1, 1);
        let _color_field: Field<BoundedU32<U0, U0, U7>, u32> = Field::new(3, 2);
    }
}
