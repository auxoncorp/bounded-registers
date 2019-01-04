use core::marker::PhantomData;
use core::ops::{BitAnd, Shr};

use typenum::consts::B1;
use typenum::{IsGreater, Unsigned};

use type_bounds::num::{Bounded, ReifyTo};

#[derive(Clone, Copy)]
pub struct ROField<N: Clone + Copy + PartialOrd, M, O, L, U>
where
    L: ReifyTo<N>,
    U: ReifyTo<N>,

    U: IsGreater<L, Output = B1>,
{
    _mask: PhantomData<M>,
    _offset: PhantomData<O>,
    val: Bounded<N, L, U>,
}

impl<N: PartialOrd, M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned>
    ROField<N, M, O, L, U>
where
    N: Clone + Copy + PartialOrd,
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

pub trait RORegister {
    type Output;
    unsafe fn get_ptr(&self) -> *const Self::Output;

    fn read<M: Unsigned, O: Unsigned, L: Unsigned, U: Unsigned>(
        &self,
        _field: ROField<Self::Output, M, O, L, U>,
    ) -> Option<ROField<Self::Output, M, O, L, U>>
    where
        L: ReifyTo<Self::Output>,
        U: ReifyTo<Self::Output> + IsGreater<L, Output = B1>,
        M: ReifyTo<Self::Output>,
        O: ReifyTo<Self::Output>,

        <Self as RORegister>::Output:
            Copy + Clone + PartialOrd + BitAnd<<Self as RORegister>::Output>,

        <<Self as RORegister>::Output as BitAnd>::Output:
            Shr<Self::Output, Output = Self::Output>,
        <<<Self as RORegister>::Output as BitAnd>::Output as Shr<
            Self::Output,
        >>::Output: Clone + Copy + PartialOrd,
    {
        let val = unsafe { *self.get_ptr() };
        ROField::new((val & M::reify()) >> O::reify())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use typenum::consts::{U0, U2, U28, U7};

    struct ROStatus {
        val: *const u8,
    }

    impl RORegister for ROStatus {
        type Output = u8;

        unsafe fn get_ptr(&self) -> *const Self::Output {
            self.val
        }
    }

    #[test]
    fn test_read_only() {
        // the value our register currently has.
        let x = 8_u8;
        let x_ptr = &x as *const u8;

        // An arbitrary field. This will have an alias like
        // Status::Color::Read
        let field: ROField<u8, U28, U2, U0, U7> = ROField::new(0).unwrap();

        // Our register and its value / ptr.
        let ror = ROStatus { val: x_ptr };

        // extracting the value of the field.
        let field_val = ror.read(field).unwrap().val();
        assert_eq!(field_val, 2);
    }
}
