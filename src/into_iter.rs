use core::{
    iter::FusedIterator,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::ArrayVec;

pub type ArrayVecIntoIter<T, const N: usize> = private::ArrayVecIntoIter<[MaybeUninit<T>; N]>;
pub type SliceVecIntoIter<T> = private::ArrayVecIntoIter<[MaybeUninit<T>]>;

mod private {
    use crate::private::ArrayVecBacking;

    pub struct ArrayVecIntoIter<Array: ArrayVecBacking + ?Sized> {
        pub(super) start: usize,
        pub(super) length: usize,
        pub(super) array: Array,
    }

    impl<Array: ArrayVecBacking + ?Sized> Drop for ArrayVecIntoIter<Array> {
        fn drop(&mut self) {
            unsafe { self.array.drop_elements(self.start, self.length) }
        }
    }
}

impl<T, const N: usize> ArrayVecIntoIter<T, N> {
    pub(super) const fn new(length: usize, array: [MaybeUninit<T>; N]) -> Self {
        Self {
            start: 0,
            length,
            array,
        }
    }

    pub const fn as_slice_vec_into_iter(&self) -> &SliceVecIntoIter<T> {
        self
    }

    pub const fn as_mut_slice_vec_into_iter(&mut self) -> &mut SliceVecIntoIter<T> {
        self
    }
}

impl<T, const N: usize> Deref for ArrayVecIntoIter<T, N> {
    type Target = SliceVecIntoIter<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T, const N: usize> DerefMut for ArrayVecIntoIter<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

impl<T, const N: usize> Clone for ArrayVecIntoIter<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut result = ArrayVec::new();
        for value in self.as_slice() {
            unsafe { result.push_unchecked(value.clone()) };
        }
        result.into_iter()
    }
}

impl<T, const N: usize> Iterator for ArrayVecIntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        (**self).next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    fn count(self) -> usize {
        self.length
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth(n)
    }
}

impl<T, const N: usize> DoubleEndedIterator for ArrayVecIntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (**self).next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth_back(n)
    }
}

impl<T, const N: usize> ExactSizeIterator for ArrayVecIntoIter<T, N> {}

impl<T, const N: usize> FusedIterator for ArrayVecIntoIter<T, N> {}

impl<T> SliceVecIntoIter<T> {
    pub const fn as_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(
                self.array.as_ptr().cast::<T>().add(self.start),
                self.length,
            )
        }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.array.as_mut_ptr().cast::<T>().add(self.start),
                self.length,
            )
        }
    }
}

impl<T> Iterator for SliceVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }

        let value = unsafe { self.array.get_unchecked(self.start).assume_init_read() };
        self.start += 1;
        self.length -= 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length, Some(self.length))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let to_drop_length = self.length.min(n);
        let to_drop = unsafe {
            core::ptr::slice_from_raw_parts_mut(
                self.array.as_mut_ptr().cast::<T>().add(self.start),
                to_drop_length,
            )
        };
        self.start += to_drop_length;
        self.length -= to_drop_length;
        unsafe { core::ptr::drop_in_place(to_drop) };
        self.next()
    }
}

impl<T> DoubleEndedIterator for SliceVecIntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }

        self.length -= 1;
        Some(unsafe {
            self.array
                .get_unchecked(self.start + self.length)
                .assume_init_read()
        })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let to_drop_length = self.length.min(n);
        self.length -= to_drop_length;
        let to_drop = unsafe {
            core::ptr::slice_from_raw_parts_mut(
                self.array
                    .as_mut_ptr()
                    .cast::<T>()
                    .add(self.start + self.length),
                to_drop_length,
            )
        };
        unsafe { core::ptr::drop_in_place(to_drop) };
        self.next_back()
    }
}

impl<T> ExactSizeIterator for SliceVecIntoIter<T> {}

impl<T> FusedIterator for SliceVecIntoIter<T> {}
