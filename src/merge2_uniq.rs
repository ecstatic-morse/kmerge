use std::cmp::Ordering;

pub fn naive<T: Ord>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<T> {
    a.append(&mut b);
    a.sort_unstable();
    a.dedup();
    a
}

pub fn into_iter<T: Ord>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    // If one of the lists is zero-length, we don't need to do any work.
    if a.is_empty() {
        return b;
    }
    if b.is_empty() {
        return a;
    }

    // Ensure that `out` always has sufficient capacity.
    //
    // SAFETY: The calls to `push_unchecked` below are safe because of this.
    let mut out = Vec::with_capacity(a.len() + b.len());

    let mut a = a.into_iter();
    let mut b = b.into_iter();

    // While both inputs have elements remaining, copy the lesser element to the output vector.
    while !a.is_empty() && !b.is_empty() {
        // SAFETY: The following calls to `get_unchecked` and `next_unchecked` are safe because we
        // ensure that `a.len() > 0` and `b.len() > 0` inside the loop.
        //
        // I was hoping to avoid using "unchecked" operations, but it seems the bounds checks
        // don't get optimized away. Using `ExactSizeIterator::is_empty` instead of checking `len`
        // seemed to help, but that method is unstable.

        let a_elem = unsafe { a.as_slice().get_unchecked(0) };
        let b_elem = unsafe { b.as_slice().get_unchecked(0) };
        match a_elem.cmp(b_elem) {
            Ordering::Less => unsafe { push_unchecked(&mut out, next_unchecked(&mut a)) },
            Ordering::Greater => unsafe { push_unchecked(&mut out, next_unchecked(&mut b)) },
            Ordering::Equal => unsafe {
                push_unchecked(&mut out, next_unchecked(&mut a));
                std::mem::drop(next_unchecked(&mut b));
            },
        }
    }

    // Once either `a` or `b` runs out of elements, copy all remaining elements in the other one
    // directly to the back of the output list.
    //
    // This branch is free because we have to check `a.is_empty()` above anyways.
    //
    // Calling `push_unchecked` in a loop was slightly faster than `out.extend(...)`
    // despite the fact that `std::vec::IntoIter` implements `TrustedLen`.
    if !a.is_empty() {
        for elem in a {
            unsafe {
                push_unchecked(&mut out, elem);
            }
        }
    } else {
        for elem in b {
            unsafe {
                push_unchecked(&mut out, elem);
            }
        }
    }

    out
}

pub fn into_iter_safer<T: Ord>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    // If one of the lists is zero-length, we don't need to do any work.
    if a.is_empty() {
        return b;
    }
    if b.is_empty() {
        return a;
    }

    // Ensure that `out` always has sufficient capacity.
    //
    // SAFETY: The calls to `push_unchecked` below are safe because of this.
    let mut out = Vec::with_capacity(a.len() + b.len());

    let mut a = a.into_iter();
    let mut b = b.into_iter();

    // While both inputs have elements remaining, copy the lesser element to the output vector.
    while !a.is_empty() && !b.is_empty() {
        // SAFETY: The following calls to `get_unchecked` and `next_unchecked` are safe because we
        // ensure that `a.len() > 0` and `b.len() > 0` inside the loop.
        //
        // I was hoping to avoid using "unchecked" operations, but it seems the bounds checks
        // don't get optimized away. Using `ExactSizeIterator::is_empty` instead of checking `len`
        // seemed to help, but that method is unstable.

        let a_elem = &a.as_slice()[0];
        let b_elem = &b.as_slice()[0];
        match a_elem.cmp(b_elem) {
            Ordering::Less => unsafe { push_unchecked(&mut out, a.next().unwrap()) },
            Ordering::Greater => unsafe { push_unchecked(&mut out, b.next().unwrap()) },
            Ordering::Equal => unsafe {
                push_unchecked(&mut out, a.next().unwrap());
                std::mem::drop(b.next().unwrap());
            },
        }
    }

    // Once either `a` or `b` runs out of elements, copy all remaining elements in the other one
    // directly to the back of the output list.
    //
    // This branch is free because we have to check `a.is_empty()` above anyways.
    if !a.is_empty() {
        out.extend(a);
    } else {
        out.extend(b);
    }

    out
}

/// Pushes `value` to `vec` without checking that the vector has sufficient capacity.
///
/// If `vec.len() == vec.cap()`, calling this function is UB.
unsafe fn push_unchecked<T>(vec: &mut Vec<T>, value: T) {
    let end = vec.as_mut_ptr().add(vec.len());
    std::ptr::write(end, value);
    vec.set_len(vec.len() + 1);
}

/// Equivalent to `iter.next().unwrap()` that is UB to call when `iter` is empty.
unsafe fn next_unchecked<T>(iter: &mut std::vec::IntoIter<T>) -> T {
    match iter.next() {
        Some(x) => x,
        None => std::hint::unreachable_unchecked(),
    }
}

pub fn old_datafrog<T: Ord>(mut a: Vec<T>, mut b: Vec<T>) -> Vec<T> {
    if a.is_empty() {
        return b;
    }
    if b.is_empty() {
        return a;
    }

    // Make sure that a starts with the lower element
    // Will not panic since both collections must have at least 1 element at this point
    if a[0] > b[0] {
        std::mem::swap(&mut a, &mut b);
    }

    let mut out = Vec::with_capacity(a.len() + b.len());
    let mut a = a.drain(..);
    let mut b = b.drain(..).peekable();

    out.push(a.next().unwrap());
    if out.first() == b.peek() {
        b.next();
    }

    for elem in a {
        while b.peek().map(|x| x.cmp(&elem)) == Some(Ordering::Less) {
            out.push(b.next().unwrap());
        }
        if b.peek().map(|x| x.cmp(&elem)) == Some(Ordering::Equal) {
            b.next();
        }
        out.push(elem);
    }

    // Finish draining second list
    out.extend(b);
    out
}

struct RawIter<T> {
    start: *mut T,
    end: *mut T,
}

impl<T> RawIter<T> {
    fn is_empty(&self) -> bool {
        self.start == self.end
    }

    fn len(&self) -> usize {
        unsafe { self.end.offset_from(self.start) as usize }
    }

    unsafe fn advance(&mut self) {
        self.start = self.start.add(1);
    }
}

pub fn raw_ptr<T: Ord>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    if a.is_empty() {
        return b;
    }
    if b.is_empty() {
        return a;
    }

    let (aptr, alen, acap) = a.into_raw_parts();
    let (bptr, blen, bcap) = b.into_raw_parts();

    let mut ait = RawIter {
        start: aptr,
        end: unsafe { aptr.add(alen) },
    };

    let mut bit = RawIter {
        start: bptr,
        end: unsafe { bptr.add(blen) },
    };

    let mut out: Vec<T> = Vec::with_capacity(alen + blen);
    let mut o = out.as_mut_ptr();

    // While elements remain in both `a` and `b`.
    while !ait.is_empty() && !bit.is_empty() {
        let ord = unsafe { (*ait.start).cmp(&*bit.start) };
        match ord {
            // a[i] < b[j]: o[k++] = a[i++]
            Ordering::Less => unsafe {
                std::ptr::copy_nonoverlapping(ait.start, o, 1);
                ait.advance();
                o = o.add(1);
            },

            // a[i] > b[j]: o[k++] = b[j++]
            Ordering::Greater => unsafe {
                std::ptr::copy_nonoverlapping(bit.start, o, 1);
                bit.advance();
                o = o.add(1);
            },

            // a[i] == b[j]: o[k++] = a[i++]; drop(b[j++])
            Ordering::Equal => unsafe {
                std::ptr::copy_nonoverlapping(ait.start, o, 1);
                ait.advance();

                // Drop the duplicate element, since it is not copied to the output vector.
                std::ptr::drop_in_place(bit.start);
                bit.advance();

                o = o.add(1);
            },
        }
    }

    unsafe {
        // Once either `a` or `b` runs out of elements, move all remaining elements in the other
        // one directly to the back of the output list.
        //
        // NOTE: This branch is free because we have to check `ait.is_empty()` above anyways.
        if !ait.is_empty() {
            std::ptr::copy_nonoverlapping(ait.start, o, ait.len());
            o = o.add(ait.len());
        } else {
            std::ptr::copy_nonoverlapping(bit.start, o, bit.len());
            o = o.add(bit.len());
        }

        // Free the capacity for `a` and `b` but not the individual elements, since those have been
        // copied into `out`.
        std::mem::drop(Vec::from_raw_parts(aptr, 0, acap));
        std::mem::drop(Vec::from_raw_parts(bptr, 0, bcap));

        // Update `out` now that all the elements have been copied.
        out.set_len(o.offset_from(out.as_ptr()) as usize);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn into_iter_impl(mut a: Vec<usize>, mut b: Vec<usize>) -> bool {
        a.sort_unstable();
        a.dedup();
        b.sort_unstable();
        b.dedup();

        let expected: Vec<_> = naive(a.clone(), b.clone());
        let actual: Vec<_> = into_iter(a, b);
        expected == actual
    }

    #[quickcheck]
    fn old_datafrog_impl(mut a: Vec<usize>, mut b: Vec<usize>) -> bool {
        a.sort_unstable();
        a.dedup();
        b.sort_unstable();
        b.dedup();

        dbg!(&a, &b);

        let expected: Vec<_> = naive(a.clone(), b.clone());
        let actual: Vec<_> = old_datafrog(a, b);
        expected == actual
    }

    #[quickcheck]
    fn raw_ptr_impl(mut a: Vec<usize>, mut b: Vec<usize>) -> bool {
        a.sort_unstable();
        a.dedup();
        b.sort_unstable();
        b.dedup();

        dbg!(&a, &b);

        let expected: Vec<_> = naive(a.clone(), b.clone());
        let actual: Vec<_> = raw_ptr(a, b);
        expected == actual
    }
}
