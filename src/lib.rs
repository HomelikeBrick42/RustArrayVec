#![no_std]
#![deny(rust_2018_idioms, rust_2024_compatibility)]

use core::{
    fmt::Debug,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Bound, Deref, DerefMut, RangeBounds},
};

#[cfg(test)]
mod tests;

mod drain;
mod into_iter;

pub use drain::*;
pub use into_iter::*;

pub struct ArrayVec<T, const CAP: usize> {
    data: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<T, const CAP: usize> ArrayVec<T, CAP> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; CAP],
            len: 0,
        }
    }

    #[must_use]
    pub const fn from_array<const N: usize>(values: [T; N]) -> Self {
        let mut array = Self::new();
        unsafe {
            const { assert!(N <= CAP) };
            let values = ManuallyDrop::new(values);
            core::ptr::copy_nonoverlapping(&raw const values, array.data.as_mut_ptr().cast(), 1);
            array.set_len(N);
        }
        array
    }

    pub const fn push(&mut self, value: T) -> Result<&mut T, T> {
        if self.len >= CAP {
            return Err(value);
        }
        unsafe {
            let ptr = self.data.as_mut_ptr().add(self.len).cast();
            self.len += 1;
            core::ptr::write(ptr, value);
            Ok(&mut *ptr)
        }
    }

    pub const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { core::ptr::read(self.data.as_ptr().add(self.len).cast()) })
        }
    }

    pub fn clear(&mut self) {
        let elements: *mut [T] = self.as_mut_slice();
        // set length before dropping elements so that panicking cant cause dropped elements to be accessed
        self.len = 0;
        unsafe { core::ptr::drop_in_place(elements) };
    }

    pub const fn insert(&mut self, index: usize, value: T) -> Result<&mut T, T> {
        if self.len < CAP && index <= self.len {
            // copy all elements to the right to make room
            unsafe {
                let ptr = self.data.as_mut_ptr().add(index);
                core::ptr::copy(ptr, ptr.add(1), self.len - index);
            }

            self.len += 1;

            // write the value at the index and return it
            unsafe {
                let ptr = self.data.as_mut_ptr().add(index).cast();
                core::ptr::write(ptr, value);
                Ok(&mut *ptr)
            }
        } else {
            Err(value)
        }
    }

    pub const fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            let element = unsafe { core::ptr::read(self.data.as_ptr().add(index).cast()) };
            self.len -= 1;

            // copy elements after the index to the left
            unsafe {
                let ptr = self.data.as_mut_ptr().add(index);
                core::ptr::copy(ptr.add(1), ptr, self.len - index);
            }

            Some(element)
        } else {
            None
        }
    }

    pub const fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            let element = unsafe { core::ptr::read(self.data.as_ptr().add(index).cast()) };
            self.len -= 1;

            if index != self.len {
                unsafe {
                    let ptr = self.data.as_mut_ptr();
                    core::ptr::copy_nonoverlapping(ptr.add(self.len), ptr.add(index), 1);
                }
            }

            Some(element)
        } else {
            None
        }
    }

    pub fn truncate(&mut self, len: usize) {
        unsafe {
            if len > self.len {
                return;
            }
            let remaining_len = self.len - len;
            let s = core::ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
            // set len before dropping incase of panics
            self.len = len;
            core::ptr::drop_in_place(s);
        }
    }

    pub const fn append<const OTHER_CAP: usize>(&mut self, other: &mut ArrayVec<T, OTHER_CAP>) {
        let new_len = self.len + other.len;
        assert!(new_len <= CAP);
        unsafe {
            core::ptr::copy_nonoverlapping(
                other.data.as_ptr(),
                self.data.as_mut_ptr().add(self.len),
                other.len,
            );
            other.set_len(0);
            self.set_len(new_len);
        }
    }

    #[must_use]
    pub fn map<F, U>(self, mut f: F) -> ArrayVec<U, CAP>
    where
        F: FnMut(T) -> U,
    {
        let mut array = ArrayVec::new();
        for element in self {
            let Ok(_) = array.push(f(element)) else {
                unsafe { core::hint::unreachable_unchecked() }
            };
        }
        array
    }

    #[must_use]
    pub fn map_ref<'a, F, U>(&'a self, mut f: F) -> ArrayVec<U, CAP>
    where
        F: FnMut(&'a T) -> U,
    {
        let mut array = ArrayVec::new();
        for element in self {
            let Ok(_) = array.push(f(element)) else {
                unsafe { core::hint::unreachable_unchecked() }
            };
        }
        array
    }

    #[must_use]
    pub fn map_mut<'a, F, U>(&'a mut self, mut f: F) -> ArrayVec<U, CAP>
    where
        F: FnMut(&'a mut T) -> U,
    {
        let mut array = ArrayVec::new();
        for element in self {
            let Ok(_) = array.push(f(element)) else {
                unsafe { core::hint::unreachable_unchecked() }
            };
        }
        array
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, CAP>
    where
        R: RangeBounds<usize>,
    {
        let len = self.len;

        let end = match range.end_bound() {
            Bound::Included(&end) => end.saturating_add(1).min(len),
            Bound::Excluded(&end) => end.min(len),
            Bound::Unbounded => len,
        };
        let start = match range.start_bound() {
            Bound::Included(&start) => start.min(len),
            Bound::Excluded(&start) => start.saturating_add(1).min(len),
            Bound::Unbounded => 0,
        }
        .min(end);

        unsafe {
            self.set_len(start);
            Drain {
                array: self,
                range_start: start,
                range_len: end - start,
                tail_start: end,
                tail_len: len - end,
            }
        }
    }

    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr().cast(), self.len) }
    }

    #[must_use]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.len) }
    }

    /// # Safety
    /// `len` must be less than or equal to `CAP`
    /// all elements 0..len must be initialized
    pub const unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= CAP);
        self.len = len;
    }
}

impl<T, const CAP: usize> Default for ArrayVec<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> Deref for ArrayVec<T, CAP> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const CAP: usize> DerefMut for ArrayVec<T, CAP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const CAP: usize> Drop for ArrayVec<T, CAP> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) }
    }
}

impl<T: Clone, const CAP: usize> Clone for ArrayVec<T, CAP> {
    fn clone(&self) -> Self {
        self.map_ref(Clone::clone)
    }

    fn clone_from(&mut self, source: &Self) {
        self.truncate(source.len);

        let (init, tail) = source.split_at(self.len);

        self.clone_from_slice(init);
        for element in tail {
            let Ok(_) = self.push(element.clone()) else {
                unsafe { core::hint::unreachable_unchecked() }
            };
        }
    }
}

impl<T: Debug, const CAP: usize> Debug for ArrayVec<T, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.as_slice(), f)
    }
}

impl<T, const CAP: usize> AsRef<[T]> for ArrayVec<T, CAP> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const CAP: usize> AsMut<[T]> for ArrayVec<T, CAP> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a ArrayVec<T, CAP> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a mut ArrayVec<T, CAP> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const CAP: usize> IntoIterator for ArrayVec<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;

    fn into_iter(self) -> Self::IntoIter {
        let this = ManuallyDrop::new(self);
        let data = unsafe { core::ptr::read(&this.data) };
        IntoIter {
            data,
            start: 0,
            end: this.len,
        }
    }
}

impl<T, const CAP: usize> FromIterator<T> for ArrayVec<T, CAP> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut array = Self::new();
        array.extend(iter);
        array
    }
}

impl<T, const CAP: usize> Extend<T> for ArrayVec<T, CAP> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|element| {
            let Ok(_) = self.push(element) else {
                panic!("ArrayVec capacity overflow")
            };
        });
    }
}
