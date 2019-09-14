use core::marker::PhantomData;
use core::ops::{Add, BitAnd, BitOr, Shl, Shr};

use typenum::consts::{True, U0};
use typenum::{IsGreater, IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use super::bounds::{Bounded, ReifyTo};

pub trait ReadOnlyRegister {
    type Width;

    /// `get_field` takes a field and sets the value of that
    /// field to its value in the register.
    fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<Self::Width, M, O, U>,
    ) -> Option<Field<Self::Width, M, O, U>>
    where
        U: IsGreater<U0, Output = True> + ReifyTo<Self::Width>,
        M: ReifyTo<Self::Width>,
        O: ReifyTo<Self::Width>,
        U0: ReifyTo<Self::Width>;

    /// `read` returns the current state of the whole register as a
    /// `Self::Width`.
    fn read(&self) -> Self::Width;

    /// `extract` pulls the state of a register out into a wrapped
    /// read-only register.
    fn extract(&self) -> ReadOnlyCopy<Self::Width>;

    /// `is_set` takes a field and returns true if that field's value
    /// is equal to its upper bound or not. This is particularly
    /// useful in single-bit fields.
    fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(&self, f: Field<Self::Width, M, O, U>) -> bool
    where
        U: IsGreater<U0, Output = True>,
        U: ReifyTo<Self::Width>,
        M: ReifyTo<Self::Width>,
        O: ReifyTo<Self::Width>;

    /// `matches_any` returns whether or not any of the given fields
    /// match those fields values inside the register.
    fn matches_any<V: Positioned<Width = Self::Width>>(&self, val: V) -> bool;

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    fn matches_all<V: Positioned<Width = Self::Width>>(&self, val: V) -> bool;
}

pub trait WriteOnlyRegister {
    type Width;

    /// `modify` takes one or more fields, joined by `+`, and
    /// sets those fields in the register, leaving the others
    /// as they were.
    fn modify<V: Positioned<Width = Self::Width>>(&mut self, val: V);

    /// `write` sets the value of the whole register to the
    /// given `Self::Width` value.
    fn write(&mut self, val: Self::Width);
}

pub trait ReadWriteRegister {
    type Width;

    /// `get_field` takes a field and sets the value of that
    /// field to its value in the register.
    fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<Self::Width, M, O, U>,
    ) -> Option<Field<Self::Width, M, O, U>>
    where
        U: IsGreater<U0, Output = True>;

    /// `read` returns the current state of the whole register as a
    /// `Self::Width`.
    fn read(&self) -> Self::Width;

    /// `extract` pulls the state of a register out into a wrapped
    /// read-only register.
    fn extract(&self) -> ReadOnlyCopy<Self::Width>;

    /// `is_set` takes a field and returns true if that field's value
    /// is equal to its upper bound or not. This is particularly
    /// useful in single-bit fields.
    fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(&self, f: Field<Self::Width, M, O, U>) -> bool
    where
        U: IsGreater<U0, Output = True>,
        U: ReifyTo<Self::Width>,
        M: ReifyTo<Self::Width>,
        O: ReifyTo<Self::Width>;

    /// `matches_any` returns whether or not any of the given fields
    /// match those fields values inside the register.
    fn matches_any<V: Positioned<Width = Self::Width>>(&self, val: V) -> bool;

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    fn matches_all<V: Positioned<Width = Self::Width>>(&self, val: V) -> bool;

    /// `modify` takes one or more fields, joined by `+`, and
    /// sets those fields in the register, leaving the others
    /// as they were.
    fn modify<V: Positioned<Width = Self::Width>>(&mut self, val: V);

    /// `write` sets the value of the whole register to the
    /// given `Self::Width` value.
    fn write(&mut self, val: Self::Width);
}

pub struct ReadOnlyCopy<W>(pub W);

impl<W> ReadOnlyRegister for ReadOnlyCopy<W>
where
    W: Copy + Clone + PartialOrd + BitAnd<W, Output = W> + Shr<W, Output = W> + Default,
{
    type Width = W;

    fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<W, M, O, U>,
    ) -> Option<Field<W, M, O, U>>
    where
        U: IsGreater<U0, Output = True> + ReifyTo<W>,
        M: ReifyTo<W>,
        O: ReifyTo<W>,
        U0: ReifyTo<W>,
    {
        f.set((self.0 & M::reify()) >> O::reify())
    }

    fn read(&self) -> W {
        self.0
    }

    fn extract(&self) -> Self {
        ReadOnlyCopy(self.0)
    }

    fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(&self, _: Field<W, M, O, U>) -> bool
    where
        U: IsGreater<U0, Output = True>,
        U: ReifyTo<W>,
        M: ReifyTo<W>,
        O: ReifyTo<W>,
    {
        ((self.0 & M::reify()) >> O::reify()) == U::reify()
    }

    fn matches_any<V: Positioned<Width = W>>(&self, val: V) -> bool {
        (val.in_position() & self.0) != W::default()
    }

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    fn matches_all<V: Positioned<Width = W>>(&self, val: V) -> bool {
        (val.in_position() & self.0) == val.in_position()
    }
}

/// A field in a register parameterized by its mask, offset, and upper
/// bound. To construct a field, its `val` must be ⩽ `U::U32`.
///
/// It uses these type-level numbers so that the mask and offset can
/// be constant.
#[derive(Debug)]
pub struct Field<W, M, O, U>
where
    U: IsGreater<U0, Output = True>,
{
    pub val: Bounded<W, U0, U>,
    pub _mask: PhantomData<M>,
    pub _offset: PhantomData<O>,
}

impl<W, M: Unsigned, O: Unsigned, U: Unsigned> Field<W, M, O, U>
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

    /// `checked` is a compile-time checked constructor for a
    /// `Field`. Its `V` parameter must be ⩽ `U`; if it is not, the
    /// program will fail to typecheck.
    pub fn checked<V: Unsigned>() -> Self
    where
        V: IsLessOrEqual<U, Output = True>,
        V: IsGreaterOrEqual<U0, Output = True>,
        V: ReifyTo<W>,
    {
        Self {
            val: Bounded::checked::<V>(),
            _offset: PhantomData,
            _mask: PhantomData,
        }
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

impl<W, M: Unsigned, O: Unsigned, U: Unsigned> PartialEq<Field<W, M, O, U>> for Field<W, M, O, U>
where
    U: IsGreater<U0, Output = True> + ReifyTo<W>,
    W: Copy + Clone + PartialOrd + BitAnd<W, Output = W> + Shr<W, Output = W> + Default,
    U0: ReifyTo<W>,
{
    fn eq(&self, rhs: &Field<W, M, O, U>) -> bool {
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

impl<W, M: Unsigned, O: Unsigned, U: Unsigned> Positioned for Field<W, M, O, U>
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
impl<W, LM: Unsigned, LO: Unsigned, LU: Unsigned, RM: Unsigned, RO: Unsigned, RU: Unsigned>
    Add<Field<W, RM, RO, RU>> for Field<W, LM, LO, LU>
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

    fn add(self, rhs: Field<W, RM, RO, RU>) -> Self::Output {
        FieldDisj {
            val: (self.val() << LO::reify()) | (rhs.val() << RO::reify()),
            mask: <LM as BitOr<RM>>::Output::reify(),
        }
    }
}

// Add where the rhs is a `FieldDisj`. This is necessary because I do
// not know which direction the compiler will associate `+`.
impl<W, M: Unsigned, O: Unsigned, U: Unsigned> Add<FieldDisj<W>> for Field<W, M, O, U>
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
impl<W, M: Unsigned, O: Unsigned, U: Unsigned> Add<Field<W, M, O, U>> for FieldDisj<W>
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

    fn add(self, rhs: Field<W, M, O, U>) -> Self::Output {
        FieldDisj {
            val: self.val | (rhs.val() << O::reify()),
            mask: self.mask | M::reify(),
        }
    }
}

pub trait Pointer {
    unsafe fn ptr(&self) -> *mut usize;
}
