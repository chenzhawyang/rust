use crate::intrinsics;
use crate::iter::{
    DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator, TrustedRandomAccess,
};
use crate::ops::Try;

/// An iterator that yields `None` forever after the underlying iterator
/// yields `None` once.
///
/// This `struct` is created by the [`fuse`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`fuse`]: trait.Iterator.html#method.fuse
/// [`Iterator`]: trait.Iterator.html
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Fuse<I> {
    // NOTE: for `I: FusedIterator`, this is always assumed `Some`!
    iter: Option<I>,
}
impl<I> Fuse<I> {
    pub(in crate::iter) fn new(iter: I) -> Fuse<I> {
        Fuse { iter: Some(iter) }
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl<I> FusedIterator for Fuse<I> where I: Iterator {}

/// Fuse the iterator if the expression is `None`.
macro_rules! fuse {
    ($self:ident . iter . $($call:tt)+) => {
        match $self.iter {
            Some(ref mut iter) => match iter.$($call)+ {
                None => {
                    $self.iter = None;
                    None
                }
                item => item,
            },
            None => None,
        }
    };
}

// NOTE: for `I: FusedIterator`, we assume that the iterator is always `Some`.
// Implementing this as a directly-expanded macro helps codegen performance.
macro_rules! unchecked {
    ($self:ident) => {
        match $self {
            Fuse { iter: Some(iter) } => iter,
            // SAFETY: the specialized iterator never sets `None`
            Fuse { iter: None } => unsafe { intrinsics::unreachable() },
        }
    };
}

// Any implementation here is made internal to avoid exposing default fns outside this trait
#[stable(feature = "rust1", since = "1.0.0")]
impl<I> Iterator for Fuse<I>
where
    I: Iterator,
{
    type Item = <I as Iterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        FuseIteratorImpl::next(self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        FuseIteratorImpl::nth(self, n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        FuseIteratorImpl::last(self)
    }

    #[inline]
    fn count(self) -> usize {
        FuseIteratorImpl::count(self)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        FuseIteratorImpl::size_hint(self)
    }

    #[inline]
    fn try_fold<Acc, Fold, R>(&mut self, acc: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>,
    {
        FuseIteratorImpl::try_fold(self, acc, fold)
    }

    #[inline]
    fn fold<Acc, Fold>(self, acc: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        FuseIteratorImpl::fold(self, acc, fold)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        FuseIteratorImpl::find(self, predicate)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<I> DoubleEndedIterator for Fuse<I>
where
    I: DoubleEndedIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<<I as Iterator>::Item> {
        FuseDoubleEndedIteratorImpl::next_back(self)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<<I as Iterator>::Item> {
        FuseDoubleEndedIteratorImpl::nth_back(self, n)
    }

    #[inline]
    fn try_rfold<Acc, Fold, R>(&mut self, acc: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>,
    {
        FuseDoubleEndedIteratorImpl::try_rfold(self, acc, fold)
    }

    #[inline]
    fn rfold<Acc, Fold>(self, acc: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        FuseDoubleEndedIteratorImpl::rfold(self, acc, fold)
    }

    #[inline]
    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        FuseDoubleEndedIteratorImpl::rfind(self, predicate)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<I> ExactSizeIterator for Fuse<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        FuseExactSizeIteratorImpl::len(self)
    }

    fn is_empty(&self) -> bool {
        FuseExactSizeIteratorImpl::is_empty(self)
    }
}

unsafe impl<I> TrustedRandomAccess for Fuse<I>
where
    I: TrustedRandomAccess,
{
    unsafe fn get_unchecked(&mut self, i: usize) -> I::Item {
        match self.iter {
            Some(ref mut iter) => iter.get_unchecked(i),
            // SAFETY: the caller asserts there is an item at `i`, so we're not exhausted.
            None => intrinsics::unreachable(),
        }
    }

    fn may_have_side_effect() -> bool {
        I::may_have_side_effect()
    }
}

// Fuse specialization trait
// Iterators and DoubleEndedIterators cannot be overlapped successfully
// So, they're separated into each it's own trait to provide internal implementations
// Similarly, ExactSizeIterators cannot be overlapped, so requires its own trait
#[doc(hidden)]
trait FuseIteratorImpl<I> {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn nth(&mut self, n: usize) -> Option<Self::Item>;
    fn last(self) -> Option<Self::Item>;
    fn count(self) -> usize;
    fn size_hint(&self) -> (usize, Option<usize>);
    fn try_fold<Acc, Fold, R>(&mut self, acc: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>;
    fn fold<Acc, Fold>(self, acc: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc;
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool;
}

// General Fuse impl
#[doc(hidden)]
impl<I> FuseIteratorImpl<I> for Fuse<I>
where
    I: Iterator,
{
    type Item = <I as Iterator>::Item;

    #[inline]
    default fn next(&mut self) -> Option<<I as Iterator>::Item> {
        fuse!(self.iter.next())
    }

    #[inline]
    default fn nth(&mut self, n: usize) -> Option<I::Item> {
        fuse!(self.iter.nth(n))
    }

    #[inline]
    default fn last(self) -> Option<I::Item> {
        match self.iter {
            Some(iter) => iter.last(),
            None => None,
        }
    }

    #[inline]
    default fn count(self) -> usize {
        match self.iter {
            Some(iter) => iter.count(),
            None => 0,
        }
    }

    #[inline]
    default fn size_hint(&self) -> (usize, Option<usize>) {
        match self.iter {
            Some(ref iter) => iter.size_hint(),
            None => (0, Some(0)),
        }
    }

    #[inline]
    default fn try_fold<Acc, Fold, R>(&mut self, mut acc: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>,
    {
        if let Some(ref mut iter) = self.iter {
            acc = iter.try_fold(acc, fold)?;
            self.iter = None;
        }
        Try::from_ok(acc)
    }

    #[inline]
    default fn fold<Acc, Fold>(self, mut acc: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        if let Some(iter) = self.iter {
            acc = iter.fold(acc, fold);
        }
        acc
    }

    #[inline]
    default fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        fuse!(self.iter.find(predicate))
    }
}

#[doc(hidden)]
impl<I> FuseIteratorImpl<I> for Fuse<I>
where
    I: FusedIterator,
{
    #[inline]
    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        unchecked!(self).next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        unchecked!(self).nth(n)
    }

    #[inline]
    fn last(self) -> Option<I::Item> {
        unchecked!(self).last()
    }

    #[inline]
    fn count(self) -> usize {
        unchecked!(self).count()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        unchecked!(self).size_hint()
    }

    #[inline]
    fn try_fold<Acc, Fold, R>(&mut self, init: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>,
    {
        unchecked!(self).try_fold(init, fold)
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        unchecked!(self).fold(init, fold)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        unchecked!(self).find(predicate)
    }
}

#[doc(hidden)]
trait FuseDoubleEndedIteratorImpl<I> {
    type Item;
    fn next_back(&mut self) -> Option<Self::Item>;
    fn nth_back(&mut self, n: usize) -> Option<Self::Item>;
    fn try_rfold<Acc, Fold, R>(&mut self, acc: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>;
    fn rfold<Acc, Fold>(self, acc: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc;
    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool;
}

#[doc(hidden)]
impl<I> FuseDoubleEndedIteratorImpl<I> for Fuse<I>
where
    I: DoubleEndedIterator,
{
    type Item = <I as Iterator>::Item;

    #[inline]
    default fn next_back(&mut self) -> Option<<I as Iterator>::Item> {
        fuse!(self.iter.next_back())
    }

    #[inline]
    default fn nth_back(&mut self, n: usize) -> Option<<I as Iterator>::Item> {
        fuse!(self.iter.nth_back(n))
    }

    #[inline]
    default fn try_rfold<Acc, Fold, R>(&mut self, mut acc: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>,
    {
        if let Some(ref mut iter) = self.iter {
            acc = iter.try_rfold(acc, fold)?;
            self.iter = None;
        }
        Try::from_ok(acc)
    }

    #[inline]
    default fn rfold<Acc, Fold>(self, mut acc: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        if let Some(iter) = self.iter {
            acc = iter.rfold(acc, fold);
        }
        acc
    }

    #[inline]
    default fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        fuse!(self.iter.rfind(predicate))
    }
}

#[doc(hidden)]
impl<I> FuseDoubleEndedIteratorImpl<I> for Fuse<I>
where
    I: DoubleEndedIterator + FusedIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<<I as Iterator>::Item> {
        unchecked!(self).next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<<I as Iterator>::Item> {
        unchecked!(self).nth_back(n)
    }

    #[inline]
    fn try_rfold<Acc, Fold, R>(&mut self, init: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: Try<Ok = Acc>,
    {
        unchecked!(self).try_rfold(init, fold)
    }

    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        unchecked!(self).rfold(init, fold)
    }

    #[inline]
    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        unchecked!(self).rfind(predicate)
    }
}

#[doc(hidden)]
trait FuseExactSizeIteratorImpl<I> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<I> FuseExactSizeIteratorImpl<I> for Fuse<I>
where
    I: ExactSizeIterator,
{
    default fn len(&self) -> usize {
        match self.iter {
            Some(ref iter) => iter.len(),
            None => 0,
        }
    }

    default fn is_empty(&self) -> bool {
        match self.iter {
            Some(ref iter) => iter.is_empty(),
            None => true,
        }
    }
}

impl<I> FuseExactSizeIteratorImpl<I> for Fuse<I>
where
    I: ExactSizeIterator + FusedIterator,
{
    fn len(&self) -> usize {
        unchecked!(self).len()
    }

    fn is_empty(&self) -> bool {
        unchecked!(self).is_empty()
    }
}
