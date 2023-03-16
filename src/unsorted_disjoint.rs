// !!!cmk make the names consistent, start/lower vs end/upper/end/...
// !!!cmk replace OptionRange with Option<RangeInclusive<T>>

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, NotIter, SortedDisjoint,
    SortedDisjointIterator, SortedStarts,
};
use num_traits::Zero;
use std::{
    cmp::{max, min},
    ops::{self, RangeInclusive},
};

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    iter: I,
    option_range: Option<RangeInclusive<T>>,
    min_value_plus_2: T,
    two: T,
}

impl<T, I> From<I> for UnsortedDisjoint<T, I::IntoIter>
where
    T: Integer,
    I: IntoIterator<Item = RangeInclusive<T>>, // Any iterator is fine
{
    fn from(into_iter: I) -> Self {
        UnsortedDisjoint {
            iter: into_iter.into_iter(),
            option_range: None,
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
        }
    }
}

impl<T, I> Iterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(range) = self.iter.next() {
                let (next_start, next_end) = range.into_inner();
                if next_start > next_end {
                    continue;
                }
                assert!(next_end <= T::max_value2()); // !!!cmk0 raise error on panic?
                if let Some(self_range) = self.option_range.clone() {
                    let (self_start, self_end) = self_range.into_inner();
                    if (next_start >= self.min_value_plus_2 && self_end <= next_start - self.two)
                        || (self_start >= self.min_value_plus_2
                            && next_end <= self_start - self.two)
                    {
                        let result = Some(self_start..=self_end);
                        self.option_range = Some(next_start..=next_end);
                        return result;
                    } else {
                        self.option_range =
                            Some(min(self_start, next_start)..=max(self_end, next_end));
                        continue;
                    }
                } else {
                    self.option_range = Some(next_start..=next_end);
                    continue;
                }
            } else if let Some(range) = self.option_range.clone() {
                self.option_range = None;
                return Some(range);
            } else {
                return None;
            }
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        if self.option_range.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    iter: I,
    len: <T as Integer>::SafeLen,
}

// cmk Rule there is no reason From's should be into iterators
impl<T: Integer, I> From<I> for SortedDisjointWithLenSoFar<T, I::IntoIter>
where
    I: IntoIterator<Item = RangeInclusive<T>>,
    I::IntoIter: SortedDisjoint,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFar {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
        }
    }
}

impl<T: Integer, I> SortedDisjointWithLenSoFar<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

impl<T: Integer, I> Iterator for SortedDisjointWithLenSoFar<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = self.iter.next() {
            let (start, end) = range.clone().into_inner();
            debug_assert!(start <= end && end <= T::max_value2());
            self.len += T::safe_len(&range);
            Some((start, end))
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<T: Integer, I> SortedDisjoint for SortedDisjointWithLenSoFar<T, I> where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint
{
}
impl<T: Integer, I> SortedStarts for SortedDisjointWithLenSoFar<T, I> where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint
{
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]

/// Give any iterator of ranges the [`SortedStarts`] trait without any checking.
pub struct AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub(crate) iter: I,
}

impl<T: Integer, I> SortedStarts for AssumeSortedStarts<T, I> where
    I: Iterator<Item = RangeInclusive<T>>
{
}

impl<T, I> AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub fn new(iter: I) -> Self {
        AssumeSortedStarts { iter }
    }
}

impl<T, I> Iterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    // !!!cmk rule add a size hint, but think about if it is correct with respect to other fields
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]

/// Give any iterator of ranges the [`SortedDisjoint`] trait without any checking.
///
/// # Example
///
/// ```
/// use range_set_int::{CheckSortedDisjoint, SortedDisjointIterator};
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new([2..=6].into_iter());
/// let c = a | b;
/// assert_eq!(c.to_string(), "1..=100");
/// ```
pub struct CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub(crate) iter: I,
    prev_end: Option<T>,
    seen_none: bool,
}

impl<T, I> CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub fn new(iter: I) -> Self {
        CheckSortedDisjoint {
            iter,
            prev_end: None,
            seen_none: false,
        }
    }
}

impl<T, I> Iterator for CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    //cmk coverage test every panic
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();
        if let Some(range) = next.as_ref() {
            assert!(
                !self.seen_none,
                "iterator cannot return Some after returning None"
            );
            let (start, end) = range.clone().into_inner();
            assert!(start < end, "start must be less than end");
            assert!(
                end <= T::max_value2(),
                "end must be less than or equal to max_value2"
            );
            //cmk give max_value2 a better name and do a text search
            if let Some(prev_end) = self.prev_end {
                assert!(
                    prev_end < T::max_value2() && prev_end + T::one() < start,
                    "ranges must be disjoint"
                );
            }
            self.prev_end = Some(end);
        } else {
            self.seen_none = true;
        }
        next
    }

    // !!!cmk rule add a size hint, but think about if it is correct with respect to other fields
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, I> ops::Not for CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        NotIter::new(self)
    }
}

impl<T: Integer, R, L> ops::BitOr<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitor(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitand(self, rhs)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::sub(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, R>;

    fn bitxor(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitxor(self, rhs)
    }
}
