use core::marker::PhantomData;
use core::ops::{Add, BitAnd, BitOr, Shl, Shr};

use typenum::consts::{True, U0};
use typenum::{IsGreater, IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use super::bounds::{Bounded, ReifyTo};

pub struct ReadOnlyCopy<W, R>(pub W, pub PhantomData<R>);

impl<W, R> ReadOnlyCopy<W, R>
where
    W: Copy + Clone + PartialOrd + BitAnd<W, Output = W> + Shr<W, Output = W> + Default,
{
    pub fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<W, M, O, U, R>,
    ) -> Option<Field<W, M, O, U, R>>
    where
        U: IsGreater<U0, Output = True> + ReifyTo<W>,
        M: ReifyTo<W>,
        O: ReifyTo<W>,
        U0: ReifyTo<W>,
    {
        f.set((self.0 & M::reify()) >> O::reify())
    }

    pub fn read(&self) -> W {
        self.0
    }

    pub fn extract(&self) -> Self {
        ReadOnlyCopy(self.0, PhantomData)
    }

    pub fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(&self, _: Field<W, M, O, U, R>) -> bool
    where
        U: IsGreater<U0, Output = True>,
        U: ReifyTo<W>,
        M: ReifyTo<W>,
        O: ReifyTo<W>,
    {
        ((self.0 & M::reify()) >> O::reify()) == U::reify()
    }

    pub fn matches_any<V: Positioned<Width = W>>(&self, val: V) -> bool {
        (val.in_position() & self.0) != W::default()
    }

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    pub fn matches_all<V: Positioned<Width = W>>(&self, val: V) -> bool {
        (val.in_position() & self.0) == val.in_position()
    }
}

/// A field in a register parameterized by its mask, offset, and upper
/// bound. To construct a field, its `val` must be ⩽ `U::U32`.
///
/// It uses these type-level numbers so that the mask and offset can
/// be constant.
#[derive(Debug)]
pub struct Field<W, M, O, U, R>
where
    U: IsGreater<U0, Output = True>,
{
    val: Bounded<W, U0, U>,
    _mask: PhantomData<M>,
    _offset: PhantomData<O>,
    _reg_type: PhantomData<R>,
}

impl<W, M: Unsigned, O: Unsigned, U: Unsigned, R> Field<W, M, O, U, R>
where
    U: IsGreater<U0, Output = True> + ReifyTo<W>,
    W: Copy + Clone + PartialOrd + BitAnd<W, Output = W> + Shr<W, Output = W> + Default,
    U: ReifyTo<W>,
    U0: ReifyTo<W>,
{
    /// New returns a `Some(Field)` if the given value is less than or equal to
    /// its upper bound, otherwise it returns `None`.
    pub fn new(val: W) -> Option<Self> {
        Bounded::new(val).map(|val| Self {
            val: val,
            _offset: PhantomData,
            _mask: PhantomData,
            _reg_type: PhantomData,
        })
    }

    /// `set` takes an existing field sets its value to `val`. If val
    /// is _not_ ⩽ `U`, it returns `None`.
    pub fn set(mut self, val: W) -> Option<Self> {
        Bounded::new(val).map(|val| {
            self.val = val;
            self
        })
    }

    /// `val` retrieves the value from the field.
    pub fn val(&self) -> W {
        self.val.val
    }

    /// `is_set` returns whether or not the field's val is equal to
    /// its upper bound.
    pub fn is_set(&self) -> bool {
        self.val.val == U::reify()
    }
}

macro_rules! checked {
    ($num_type:ty) => {
        impl<M: Unsigned, O: Unsigned, U: Unsigned, R> Field<$num_type, M, O, U, R>
        where
            U: IsGreater<U0, Output = True>,
        {
            /// `checked` is a compile-time checked constructor for a
            /// `Field`. Its `V` parameter must be ⩽ `U`; if it is not, the
            /// program will fail to typecheck.
            pub const fn checked<V: Unsigned>() -> Self
            where
                V: IsLessOrEqual<U, Output = True>,
                V: IsGreaterOrEqual<U0, Output = True>,
            {
                Self {
                    val: Bounded::<$num_type, U0, U>::checked::<V>(),
                    _offset: PhantomData,
                    _mask: PhantomData,
                    _reg_type: PhantomData,
                }
            }
        }
    };
}

checked!(u8);
checked!(u16);
checked!(u32);
checked!(u64);
checked!(usize);

impl<W, M: Unsigned, O: Unsigned, U: Unsigned, R> PartialEq<Field<W, M, O, U, R>>
    for Field<W, M, O, U, R>
where
    U: IsGreater<U0, Output = True> + ReifyTo<W>,
    W: Copy + Clone + PartialOrd + BitAnd<W, Output = W> + Shr<W, Output = W> + Default,
    U0: ReifyTo<W>,
{
    fn eq(&self, rhs: &Field<W, M, O, U, R>) -> bool {
        self.val() == rhs.val()
    }
}

/// `Positioned` is a trait which is used to represent a value, be it
/// a `Field` or simply a `u32`, as its value were it to be _in
/// position_ in its register.
///
/// It comes into play in `Register::modify` where, in the case of a
/// use like `Field1 + Field2 + Field3`, it is simply a no-op; as the
/// `+` implementation already positions the field. On the other hand,
/// when simply passing one `Field`, `in_position` will shift the
/// `Field`'s value right by `O`.
pub trait Positioned {
    type Width;
    fn mask(&self) -> Self::Width;
    fn in_position(&self) -> Self::Width;
}

impl<W, M: Unsigned, O: Unsigned, U: Unsigned, R> Positioned for Field<W, M, O, U, R>
where
    U: IsGreater<U0, Output = True> + ReifyTo<W>,
    W: Copy
        + Clone
        + PartialOrd
        + BitAnd<W, Output = W>
        + Shr<W, Output = W>
        + Default
        + Shl<W, Output = W>,
    M: ReifyTo<W>,
    U0: ReifyTo<W>,
    O: ReifyTo<W>,
{
    type Width = W;

    /// The mask for this positioned value.
    fn mask(&self) -> W {
        M::reify()
    }

    /// Presents a value as its register-relative value.
    fn in_position(&self) -> W {
        self.val() << O::reify()
    }
}

/// `FieldDisj` is short for _Field Disjunction_. It is a type which
/// constitutes the intermediate result of the summing, or disjunct of
/// two fields. It is not a type which one should use directly.
pub struct FieldDisj<W> {
    mask: W,
    val: W,
}

impl<W: Copy> Positioned for FieldDisj<W> {
    type Width = W;

    fn mask(&self) -> W {
        self.mask
    }

    fn in_position(&self) -> W {
        self.val
    }
}

// Add where both lhs and rhs are `Field`s.
impl<
        W,
        LM: Unsigned,
        LO: Unsigned,
        LU: Unsigned,
        LR,
        RM: Unsigned,
        RO: Unsigned,
        RU: Unsigned,
        RR,
    > Add<Field<W, RM, RO, RU, RR>> for Field<W, LM, LO, LU, LR>
where
    LU: IsGreater<U0, Output = True> + ReifyTo<W>,
    RU: IsGreater<U0, Output = True> + ReifyTo<W>,
    RO: ReifyTo<W>,
    LO: ReifyTo<W>,
    W: Copy
        + Clone
        + PartialOrd
        + BitAnd<W, Output = W>
        + Shr<W, Output = W>
        + Default
        + Shl<W, Output = W>
        + BitOr<W, Output = W>,
    U0: ReifyTo<W>,
    LM: BitOr<RM>,
    <LM as BitOr<RM>>::Output: ReifyTo<W>,
{
    type Output = FieldDisj<W>;

    fn add(self, rhs: Field<W, RM, RO, RU, RR>) -> Self::Output {
        FieldDisj {
            val: (self.val() << LO::reify()) | (rhs.val() << RO::reify()),
            mask: <LM as BitOr<RM>>::Output::reify(),
        }
    }
}

// Add where the rhs is a `FieldDisj`. This is necessary because I do
// not know which direction the compiler will associate `+`.
impl<W, M: Unsigned, O: Unsigned, U: Unsigned, R> Add<FieldDisj<W>> for Field<W, M, O, U, R>
where
    U: IsGreater<U0, Output = True> + ReifyTo<W>,
    W: Copy
        + Clone
        + PartialOrd
        + BitAnd<W, Output = W>
        + Shr<W, Output = W>
        + Default
        + Shl<W, Output = W>
        + BitOr<W, Output = W>,
    U0: ReifyTo<W>,
    O: ReifyTo<W>,
    M: ReifyTo<W>,
{
    type Output = FieldDisj<W>;

    fn add(self, rhs: FieldDisj<W>) -> Self::Output {
        FieldDisj {
            val: (self.val() << O::reify()) | rhs.val,
            mask: M::reify() | rhs.mask(),
        }
    }
}

// Add where the lhs is a `FieldDisj`. This is necessary because I do
// not know which direction the compiler will associate `+`.
impl<W, M: Unsigned, O: Unsigned, U: Unsigned, R> Add<Field<W, M, O, U, R>> for FieldDisj<W>
where
    U: IsGreater<U0, Output = True> + ReifyTo<W>,
    W: Copy
        + Clone
        + PartialOrd
        + BitAnd<W, Output = W>
        + Shr<W, Output = W>
        + Default
        + Shl<W, Output = W>
        + BitOr<W, Output = W>,
    U0: ReifyTo<W>,
    O: ReifyTo<W>,
    M: ReifyTo<W>,
{
    type Output = FieldDisj<W>;

    fn add(self, rhs: Field<W, M, O, U, R>) -> Self::Output {
        FieldDisj {
            val: self.val | (rhs.val() << O::reify()),
            mask: self.mask | M::reify(),
        }
    }
}

pub trait Pointer {
    unsafe fn ptr(&self) -> *mut usize;
}
