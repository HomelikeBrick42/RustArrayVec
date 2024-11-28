use crate::ArrayVec;
use core::{fmt::Debug, iter::FusedIterator, mem::MaybeUninit};

pub struct IntoIter<T, const CAP: usize> {
    pub(crate) data: [MaybeUninit<T>; CAP],
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<T, const CAP: usize> IntoIter<T, CAP> {
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(
                self.data.as_ptr().add(self.start).cast(),
                self.end - self.start,
            )
        }
    }

    #[must_use]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(self.start).cast(),
                self.end - self.start,
            )
        }
    }
}

impl<T, const CAP: usize> Drop for IntoIter<T, CAP> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) }
    }
}

impl<T: Clone, const CAP: usize> Clone for IntoIter<T, CAP> {
    fn clone(&self) -> Self {
        let mut array = ArrayVec::new();
        for element in self.as_slice() {
            let Ok(_) = array.push(element.clone()) else {
                unsafe { core::hint::unreachable_unchecked() }
            };
        }
        array.into_iter()
    }
}

impl<T: Debug, const CAP: usize> Debug for IntoIter<T, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T, const CAP: usize> Iterator for IntoIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            let index = self.start;
            self.start += 1;
            Some(unsafe { self.data.get_unchecked(index).assume_init_read() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.end - self.start;
        (length, Some(length))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.end - self.start
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    // TODO: should this exist? it changes how panics drop the elements
    //
    // fn nth(&mut self, n: usize) -> Option<Self::Item> {
    //     if self.start.saturating_add(n) < self.end {
    //         unsafe {
    //             let skipped: *mut [T] = core::ptr::slice_from_raw_parts_mut(
    //                 self.data.as_mut_ptr().add(self.start).cast::<T>(),
    //                 n,
    //             );
    //             // skip elements before dropping incase of panics
    //             self.start += n;
    //             // drop all elements that are skipped
    //             core::ptr::drop_in_place(skipped);
    //         }
    //         self.next()
    //     } else {
    //         let elements: *mut [T] = self.as_mut_slice();
    //         // skip elements before dropping incase of panics
    //         self.start = self.end;
    //         unsafe { core::ptr::drop_in_place(elements) };
    //         None
    //     }
    // }
}

impl<T, const CAP: usize> DoubleEndedIterator for IntoIter<T, CAP> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            Some(unsafe { self.data.get_unchecked(self.end).assume_init_read() })
        }
    }

    // TODO: should this exist? it changes how panics drop the elements
    //
    // fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    //     if self.start < self.end.saturating_sub(n) {
    //         // drop all elements that are skipped
    //         unsafe {
    //             let skipped = core::ptr::slice_from_raw_parts_mut(
    //                 self.data.as_mut_ptr().add(self.end - n).cast::<T>(),
    //                 n,
    //             );
    //             // skip elements before dropping incase of panics
    //             self.end -= n;
    //             core::ptr::drop_in_place(skipped);
    //         }
    //         self.next_back()
    //     } else {
    //         let elements: *mut [T] = self.as_mut_slice();
    //         // skip elements before dropping incase of panics
    //         self.end = self.start;
    //         unsafe { core::ptr::drop_in_place(elements) };
    //         None
    //     }
    // }
}

impl<T, const CAP: usize> ExactSizeIterator for IntoIter<T, CAP> {}

impl<T, const CAP: usize> FusedIterator for IntoIter<T, CAP> {}
