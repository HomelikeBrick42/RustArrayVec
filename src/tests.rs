use crate::ArrayVec;

extern crate std;

#[test]
fn appending_elements() {
    let mut array = ArrayVec::<i32, 2>::new();
    array.push(5).unwrap();
    array.push(6).unwrap();
    assert_eq!(array.push(7), Err(7));
    assert_eq!(array.pop(), Some(6));
    array.push(8).unwrap();
    assert_eq!(array.remove(0), Some(5));
    assert_eq!(array.pop(), Some(8));
    assert_eq!(array.pop(), None);
}

/*
#[test]
fn into_iter() {
    count_drop!();

    let array: ArrayVec<(i32, CountDrop), 4> = ArrayVec::from_array([
        (1, CountDrop(true)),
        (2, CountDrop(true)),
        (3, CountDrop(true)),
    ]);
    let mut iter = array.into_iter();
    assert_eq!(iter.next(), Some((1, CountDrop(false))));
    drop(iter);

    assert_eq!(DROP_COUNT.get(), 3);
}

#[test]
fn drain() {
    count_drop!();

    let mut values: ArrayVec<(i32, CountDrop), 6> = ArrayVec::from_array([
        (1, CountDrop(true)),
        (2, CountDrop(true)),
        (3, CountDrop(true)),
        (4, CountDrop(true)),
        (5, CountDrop(true)),
        (6, CountDrop(true)),
    ]);
    let mut iter = values.drain(1..3);
    assert_eq!(iter.next(), Some((2, CountDrop(false))));
    assert_eq!(iter.next(), Some((3, CountDrop(false))));
    assert_eq!(iter.next(), None);
    drop(iter);
    assert_eq!(
        values.as_slice(),
        &[
            (1, CountDrop(false)),
            (4, CountDrop(false)),
            (5, CountDrop(false)),
            (6, CountDrop(false)),
        ]
    );
    values.drain(..);
    assert!(values.is_empty());

    values.push((42, CountDrop(true))).unwrap();
    values.push((43, CountDrop(true))).unwrap();
    values.push((44, CountDrop(true))).unwrap();
    values.push((45, CountDrop(true))).unwrap();
    let mut iter = values.drain(1..3);
    assert_eq!(iter.next(), Some((43, CountDrop(false))));
    assert_eq!(iter.as_slice(), &[(44, CountDrop(false))]);
    iter.keep_rest();
    assert_eq!(
        values.as_slice(),
        &[
            (42, CountDrop(false)),
            (44, CountDrop(false)),
            (45, CountDrop(false)),
        ]
    );

    drop(values);

    assert_eq!(DROP_COUNT.get(), 10);
}
macro_rules! count_drop {
    () => {
        use core::cell::Cell;

        std::thread_local! {
            static DROP_COUNT: Cell<usize> = const { Cell::new(0) };
        }

        #[derive(Debug)]
        struct CountDrop(bool);

        impl PartialEq for CountDrop {
            fn eq(&self, _other: &Self) -> bool {
                true
            }
        }

        impl Drop for CountDrop {
            fn drop(&mut self) {
                if self.0 {
                    DROP_COUNT.set(DROP_COUNT.get() + 1);
                }
            }
        }
    };
}
use count_drop;
*/
