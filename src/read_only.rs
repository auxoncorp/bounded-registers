use core::marker::PhantomData;
use core::ops::{BitAnd, Shr};

use typenum::consts::{B1, U0};
use typenum::{IsGreater, Unsigned};

use type_bounds::num::{Bounded, ReifyTo};

#[derive(Clone, Copy)]
pub struct Field<N: Clone + Copy + PartialOrd, M, O, L, U>
where
    L: ReifyTo<N>,
    U: ReifyTo<N>,

    U: IsGreater<L, Output = B1>,
{
    _mask: PhantomData<M>,
    _offset: PhantomData<O>,
    val: Bounded<N, L, U>,
}

impl<N, M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned> Field<N, M, O, L, U>
where
    N: Clone + Copy + PartialOrd + PartialEq,
    L: ReifyTo<N>,
    U: ReifyTo<N>,

    U: IsGreater<L, Output = B1>,
{
    pub fn new(val: N) -> Option<Self> {
        match Bounded::new(val) {
            Some(b) => Some(Self {
                _mask: PhantomData,
                _offset: PhantomData,
                val: b,
            }),
            None => None,
        }
    }

    pub fn val(&self) -> N {
        self.val.val
    }
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Field<u32, M, O, U0, U>
where
    U: IsGreater<U0, Output = B1>,
{
    pub const fn zero() -> Self {
        Self {
            _mask: PhantomData,
            _offset: PhantomData,
            val: Bounded::zero(),
        }
    }
}

impl<N: PartialOrd, M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned>
    PartialEq<N> for Field<N, M, O, L, U>
where
    N: Clone + Copy + PartialOrd + PartialEq,
    L: ReifyTo<N>,
    U: ReifyTo<N>,

    U: IsGreater<L, Output = B1>,
{
    fn eq(&self, rhs: &N) -> bool {
        self.val() == *rhs
    }
}

pub struct Register<N> {
    ptr: *const N,
}

impl<N> Register<N>
where
    N: Copy + PartialOrd,
{
    pub fn new(ptr: *const N) -> Self {
        Self { ptr }
    }

    pub fn read<M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned>(
        &self,
        _field: Field<N, M, O, L, U>,
    ) -> Option<Field<N, M, O, L, U>>
    where
        L: ReifyTo<N>,
        U: ReifyTo<N> + IsGreater<L, Output = B1>,
        M: ReifyTo<N>,
        O: ReifyTo<N>,

        N: BitAnd<N>,
        <N as BitAnd<N>>::Output: Shr<N, Output = N>,
    {
        let val = unsafe { *self.ptr };
        Field::new((val & M::reify()) >> O::reify())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use typenum::consts::{U0, U2, U28, U7};

    #[test]
    fn test_read_only() {
        // the value our register currently has.
        let x = 8_u8;
        let x_ptr = &x as *const u8;

        // An arbitrary field. This will have an alias like
        // Status::Color::Read
        let field: Field<u8, U28, U2, U0, U7> = Field::new(0).unwrap();

        // Our register and its value / ptr.
        let ror = Register::new(x_ptr);

        // extracting the value of the field.
        let field_val = ror.read(field).unwrap().val();
        assert_eq!(field_val, 2);
    }
}
