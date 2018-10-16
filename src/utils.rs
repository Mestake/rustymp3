use std::mem;
use std::ops::*;

pub trait SliceExt {
    fn advance(&mut self, n: usize);
}

pub trait UnsignedInteger {}

#[allow(unused)]
type TransitTy = usize;

/// This trait grantes the `bit_range` method to you. The trait:
///   - cleares all the bits outside the given range
//      and shifts the bits within range to least position
///   - Works with all the unsigned integer types
///   - Works with all the unsigned range types
///   - Limited to unsigned types only so no UB is involved
///   - Has a foolprof, no way to do something wrong
pub trait UIntBitsRng
where
    Self: Copy
        + Shl<TransitTy, Output = Self>
        + Shr<TransitTy, Output = Self>
        + UnsignedInteger,
{
    #[inline(always)]
    fn bit_range<R, T>(self, range: R) -> Self
    where
        TransitTy: From<T>,
        R: RangeBounds<T>,
        T: PartialOrd<usize>
            + PartialOrd<T>
            + Copy
            + From<TransitTy>
            + Sub<T, Output = T>,
    {
        use self::Bound::*;

        let self_sz = mem::size_of::<Self>() * 8;

        let low = match range.start_bound() {
            Unbounded => T::from(0 as TransitTy),
            Included(l) => *l,
            Excluded(_) => unreachable!(
                "starting bound cannot be an excluded one"
            ),
        };
        let hig = match range.end_bound() {
            Unbounded => T::from(0),
            Included(h) => {
                debug_assert!(*h < self_sz, "out of range");
                debug_assert!(low <= *h, "incorrect range");

                T::from(self_sz) - T::from(1) - *h
            }
            Excluded(h) => {
                debug_assert!(*h <= self_sz, "out of range");
                debug_assert!(low <= *h, "incorrect range");

                T::from(self_sz) - *h
            }
        };

        debug_assert!(low <= self_sz, "out of range");

        let low = TransitTy::from(low);
        let hig = TransitTy::from(hig);

        
        self 
            >> low << low // clear `low` lesser bits
            << hig >> hig // clear `hig` highter bits
            >> low // shift bits to the lowest position
    }
}

impl<'a, T> SliceExt for &'a [T] {
    fn advance(&mut self, n: usize) {
        *self = unsafe { mem::transmute(&self[n..]) };
    }
}

impl<'a, T> SliceExt for &'a mut [T] {
    fn advance(&mut self, n: usize) {
        *self = unsafe { mem::transmute(&mut self[n..]) };
    }
}

impl UnsignedInteger for u8 {}
impl UnsignedInteger for u16 {}
impl UnsignedInteger for u32 {}
impl UnsignedInteger for u64 {}
impl UnsignedInteger for usize {}

impl UIntBitsRng for u8 {}
impl UIntBitsRng for u16 {}
impl UIntBitsRng for u32 {}
impl UIntBitsRng for u64 {}
impl UIntBitsRng for usize {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_range() {
        let range = 0b10001100_11101111_11110111_00110001 as u32;

        assert_eq!(range.bit_range(..=7), 0b00110001);
        assert_eq!(range.bit_range(8..16), 0b11110111);
        assert_eq!(range.bit_range(16..=23), 0b11101111);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn bit_range_fail1() {
        1u8.bit_range(..=11);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn bit_range_fail2() {
        1u8.bit_range(12..13);
    }

    #[test]
    #[should_panic(expected = "incorrect range")]
    fn bit_range_fail3() {
        1u8.bit_range(2..1);
    }

    #[test]
    fn slice_advance() {
        let mut slice = &[1, 2, 3, 4][..];

        slice.advance(0);
        assert_eq!(slice, &[1, 2, 3, 4][..]);

        slice.advance(2);
        assert_eq!(slice, &[3, 4][..]);
    }

    #[test]
    #[should_panic]
    fn slice_shilf_fail() {
        (&[1u32][..]).advance(2)
    }
}
