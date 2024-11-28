use crate::ArrayVec;
use core::iter::FusedIterator;

pub struct Drain<'a, T, const CAP: usize> {
    pub(crate) array: &'a mut ArrayVec<T, CAP>,
    pub(crate) range_start: usize,
    pub(crate) range_len: usize,
    pub(crate) tail_start: usize,
    pub(crate) tail_len: usize,
}

impl<T, const CAP: usize> Drain<'_, T, CAP> {
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(
                self.array.data.as_ptr().add(self.range_start).cast(),
                self.range_len,
            )
        }
    }

    #[must_use]
    pub const fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.array.data.as_mut_ptr().add(self.range_start).cast(),
                self.range_len,
            )
        }
    }

    pub fn keep_rest(mut self) {
        let start = self.array.len;
        if start != self.range_start {
            unsafe {
                let ptr = self.array.data.as_mut_ptr();
                core::ptr::copy(ptr.add(self.range_start), ptr.add(start), self.range_len);
                self.array.len += self.range_len;
            }
        }
        self.range_len = 0;
        // the drop impl will move the tail
    }
}

impl<T, const CAP: usize> Iterator for Drain<'_, T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_len > 0 {
            let value = unsafe {
                self.array
                    .data
                    .get_unchecked(self.range_start)
                    .assume_init_read()
            };
            self.range_start += 1;
            self.range_len -= 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.range_len, Some(self.range_len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.range_len
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }
}

impl<T, const CAP: usize> DoubleEndedIterator for Drain<'_, T, CAP> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.range_len > 0 {
            self.range_len -= 1;
            Some(unsafe {
                self.array
                    .data
                    .get_unchecked(self.range_start + self.range_len)
                    .assume_init_read()
            })
        } else {
            None
        }
    }
}

impl<T, const CAP: usize> ExactSizeIterator for Drain<'_, T, CAP> {}

impl<T, const CAP: usize> FusedIterator for Drain<'_, T, CAP> {}

impl<T, const CAP: usize> Drop for Drain<'_, T, CAP> {
    fn drop(&mut self) {
        // drop all remaining elements in range
        unsafe {
            core::ptr::drop_in_place(self.as_slice_mut());
        }

        let start = self.array.len;
        if start != self.tail_start {
            unsafe {
                let ptr = self.array.data.as_mut_ptr();
                core::ptr::copy(ptr.add(self.tail_start), ptr.add(start), self.tail_len);
                self.array.len += self.tail_len;
            }
        }
    }
}
