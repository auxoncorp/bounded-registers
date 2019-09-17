use core::marker::PhantomData;

use typenum::consts::True;
use typenum::{IsGreater, IsGreaterOrEqual, IsLessOrEqual, Unsigned};

/// A type whose behaviors enforce that its `val` member fall with in
/// the range prescribed by `L` (a lower bound) and `U` (an upper
/// bound).
#[derive(Clone, Copy, Debug)]
pub struct Bounded<N, L, U> {
    pub val: N,
    _lower: PhantomData<L>,
    _upper: PhantomData<U>,
}

impl<N, L, U> Bounded<N, L, U>
where
    N: Clone + Copy + PartialOrd,
    L: ReifyTo<N>,
    U: ReifyTo<N>,
    U: IsGreater<L, Output = True>,
{
    /// Make a new instance of a bounded value. If `L <= val <= U`
    /// does not hold, then `new` returns `None`.
    pub fn new(val: N) -> Option<Self> {
        if val >= L::reify() && val <= U::reify() {
            Some(Bounded {
                val: val,
                _lower: PhantomData,
                _upper: PhantomData,
            })
        } else {
            None
        }
    }
}

macro_rules! boundeds {
    ($num_type:ty) => {
        impl<L, U> Bounded<$num_type, L, U> {
            /// Compile-type checked value.
            pub const fn checked<V: Unsigned>() -> Self
            where
                V: IsLessOrEqual<U, Output = True>,
                V: IsGreaterOrEqual<L, Output = True>,
            {
                Self {
                    val: Reifier::<V, $num_type>::reify(),
                    _lower: PhantomData,
                    _upper: PhantomData,
                }
            }
        }
    };
}

boundeds!(u8);
boundeds!(u16);
boundeds!(u32);
boundeds!(u64);
boundeds!(usize);

/// `Reify` is basically `From`, but both types are foreign so we have
/// to make a new trait. It's the last peice to our numeric-like
/// typeclass trait thingy and allows us to convert _any_ `Unsigned`
/// type to some target numeric type.
///
/// *Note*: You probably don't want to use this directly.
pub trait ReifyTo<T> {
    fn reify() -> T;
}

impl<T: Unsigned> ReifyTo<u8> for T {
    fn reify() -> u8 {
        T::U8
    }
}

impl<T: Unsigned> ReifyTo<u16> for T {
    fn reify() -> u16 {
        T::U16
    }
}

impl<T: Unsigned> ReifyTo<u32> for T {
    fn reify() -> u32 {
        T::U32
    }
}

impl<T: Unsigned> ReifyTo<usize> for T {
    fn reify() -> usize {
        T::USIZE
    }
}

/// We have to jump through some hoops to get types to
/// align. `Reifier` is a parametric version of something like `From`
/// that we can use to implement `reify()` as a const function; you'll
/// find it's used in the generated code to make the field values we
/// know ahead of time `const`.
///
/// *Note*: You probably don't want to use this directly.
pub struct Reifier<U: Unsigned, T> {
    _val: PhantomData<U>,
    _to: PhantomData<T>,
}

macro_rules! reifier {
    ($num_type:ty, $unsigned:ident) => {
        impl<U: Unsigned> Reifier<U, $num_type> {
            pub const fn reify() -> $num_type {
                U::$unsigned
            }
        }
    };
}

reifier!(u8, U8);
reifier!(u16, U16);
reifier!(u32, U32);
reifier!(u64, U64);
reifier!(usize, USIZE);

#[cfg(test)]
mod test {
    use super::*;

    use typenum::consts::{U0, U2};

    #[test]
    fn within_range() {
        let b: Bounded<u8, U0, U2> = Bounded::new(1).unwrap();
        assert_eq!(b.val, 1);
    }

    #[test]
    fn contravenes() {
        let b: Option<Bounded<u8, U0, U2>> = Bounded::new(5);
        assert!(b.is_none());
    }
}
