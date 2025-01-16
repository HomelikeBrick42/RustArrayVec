#![no_std]
#![deny(rust_2018_idioms, rust_2024_compatibility)]

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[cfg(test)]
mod tests;

pub type ArrayVec<T, const N: usize> = private::ArrayVec<[MaybeUninit<T>; N]>;
pub type SliceVec<T> = private::ArrayVec<[MaybeUninit<T>]>;

mod private {
    use core::mem::MaybeUninit;

    pub struct ArrayVec<Array: ArrayVecBacking + ?Sized> {
        pub(super) length: usize,
        pub(super) array: Array,
    }

    pub trait ArrayVecBacking {
        unsafe fn drop_elements(&mut self, length: usize);
    }

    impl<T, const N: usize> ArrayVecBacking for [MaybeUninit<T>; N] {
        unsafe fn drop_elements(&mut self, length: usize) {
            unsafe {
                core::ptr::drop_in_place(core::ptr::slice_from_raw_parts_mut(
                    self.as_mut_ptr().cast::<T>(),
                    length,
                ))
            }
        }
    }

    impl<T> ArrayVecBacking for [MaybeUninit<T>] {
        unsafe fn drop_elements(&mut self, length: usize) {
            unsafe {
                core::ptr::drop_in_place(core::ptr::slice_from_raw_parts_mut(
                    self.as_mut_ptr().cast::<T>(),
                    length,
                ))
            }
        }
    }

    impl<Array: ArrayVecBacking + ?Sized> Drop for ArrayVec<Array> {
        fn drop(&mut self) {
            unsafe { self.array.drop_elements(self.length) }
        }
    }
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub const fn new() -> Self {
        Self {
            length: 0,
            array: [const { MaybeUninit::uninit() }; N],
        }
    }

    pub const fn from_array<const M: usize>(array: [T; M]) -> Self {
        let mut result = Self::new();

        unsafe {
            let array = MaybeUninit::new(array);

            const { assert!(M <= N) };
            core::ptr::copy_nonoverlapping(array.as_ptr(), result.array.as_mut_ptr().cast(), 1);
            result.length = M;
        }

        result
    }

    pub fn map_ref<'a, U>(&'a self, mut f: impl FnMut(&'a T) -> U) -> ArrayVec<U, N> {
        let mut result = ArrayVec::new();
        for value in self {
            unsafe { result.push_unchecked(f(value)) };
        }
        result
    }

    pub fn map_mut<'a, U>(&'a mut self, mut f: impl FnMut(&'a mut T) -> U) -> ArrayVec<U, N> {
        let mut result = ArrayVec::new();
        for value in self {
            unsafe { result.push_unchecked(f(value)) };
        }
        result
    }

    pub const fn as_slice_vec(&self) -> &SliceVec<T> {
        self
    }

    pub const fn as_mut_slice_vec(&mut self) -> &mut SliceVec<T> {
        self
    }
}

impl<T, const N: usize> Clone for ArrayVec<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.map_ref(Clone::clone)
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize, const M: usize> From<[T; M]> for ArrayVec<T, N> {
    fn from(value: [T; M]) -> Self {
        Self::from_array(value)
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = SliceVec<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> SliceVec<T> {
    pub const fn push(&mut self, value: T) -> Result<&mut T, T> {
        if self.length < self.capacity() {
            Ok(unsafe { self.push_unchecked(value) })
        } else {
            Err(value)
        }
    }

    pub const unsafe fn push_unchecked(&mut self, value: T) -> &mut T {
        unsafe {
            let ptr = self.array.as_mut_ptr().add(self.length).cast::<T>();
            ptr.write(value);
            self.length += 1;
            &mut *ptr
        }
    }

    pub const fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            Some(unsafe { self.pop_unchecked() })
        } else {
            None
        }
    }

    pub const unsafe fn pop_unchecked(&mut self) -> T {
        unsafe {
            self.length -= 1;
            self.array.as_mut_ptr().add(self.length).cast::<T>().read()
        }
    }

    pub const fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.length {
            Some(unsafe { self.remove_unchecked(index) })
        } else {
            None
        }
    }

    pub const unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        unsafe {
            let ptr = self.array.as_mut_ptr().add(index);
            let value = ptr.cast::<T>().read();
            self.length -= 1;
            core::ptr::copy(ptr.add(1), ptr, self.length - index);
            value
        }
    }

    pub const fn capacity(&self) -> usize {
        self.array.len()
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.array.as_ptr().cast(), self.length) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.array.as_mut_ptr().cast(), self.length) }
    }
}

impl<T> Deref for SliceVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for SliceVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<'a, T> IntoIterator for &'a SliceVec<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SliceVec<T> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
