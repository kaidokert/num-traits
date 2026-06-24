use core::mem::size_of;
use core::num::Wrapping;
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};

c0nst::c0nst! {
/// A generic trait for converting a value to a number.
///
/// A value can be represented by the target type when it lies within
/// the range of scalars supported by the target type.
/// For example, a negative integer cannot be represented by an unsigned
/// integer type, and an `i64` with a very high magnitude might not be
/// convertible to an `i32`.
/// On the other hand, conversions with possible precision loss or truncation
/// are admitted, like an `f32` with a decimal part to an integer type, or
/// even a large `f64` saturating to `f32` infinity.
pub c0nst trait ToPrimitive {
    /// Converts the value of `self` to an `isize`. If the value cannot be
    /// represented by an `isize`, then `None` is returned.
    fn to_isize(&self) -> Option<isize> {
        match self.to_i64() { Some(v) => crate::CheckedCast::<isize>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to an `i8`. If the value cannot be
    /// represented by an `i8`, then `None` is returned.
    fn to_i8(&self) -> Option<i8> {
        match self.to_i64() { Some(v) => crate::CheckedCast::<i8>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to an `i16`. If the value cannot be
    /// represented by an `i16`, then `None` is returned.
    fn to_i16(&self) -> Option<i16> {
        match self.to_i64() { Some(v) => crate::CheckedCast::<i16>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to an `i32`. If the value cannot be
    /// represented by an `i32`, then `None` is returned.
    fn to_i32(&self) -> Option<i32> {
        match self.to_i64() { Some(v) => crate::CheckedCast::<i32>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to an `i64`. If the value cannot be
    /// represented by an `i64`, then `None` is returned.
    fn to_i64(&self) -> Option<i64>;

    /// Converts the value of `self` to an `i128`. If the value cannot be
    /// represented by an `i128`, then `None` is returned.
    fn to_i128(&self) -> Option<i128> {
        // Fall back to `to_u64` so values in `(i64::MAX, u64::MAX]` (which
        // `to_i64` rejects but `i128` represents exactly) still convert.
        match self.to_i64() {
            Some(i) => Some(i as i128),
            None => match self.to_u64() {
                Some(u) => Some(u as i128),
                None => None,
            },
        }
    }

    /// Converts the value of `self` to a `usize`. If the value cannot be
    /// represented by a `usize`, then `None` is returned.
    fn to_usize(&self) -> Option<usize> {
        match self.to_u64() { Some(v) => crate::CheckedCast::<usize>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to a `u8`. If the value cannot be
    /// represented by a `u8`, then `None` is returned.
    fn to_u8(&self) -> Option<u8> {
        match self.to_u64() { Some(v) => crate::CheckedCast::<u8>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to a `u16`. If the value cannot be
    /// represented by a `u16`, then `None` is returned.
    fn to_u16(&self) -> Option<u16> {
        match self.to_u64() { Some(v) => crate::CheckedCast::<u16>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to a `u32`. If the value cannot be
    /// represented by a `u32`, then `None` is returned.
    fn to_u32(&self) -> Option<u32> {
        match self.to_u64() { Some(v) => crate::CheckedCast::<u32>::checked_cast(v), None => None }
    }

    /// Converts the value of `self` to a `u64`. If the value cannot be
    /// represented by a `u64`, then `None` is returned.
    fn to_u64(&self) -> Option<u64>;

    /// Converts the value of `self` to a `u128`. If the value cannot be
    /// represented by a `u128`, then `None` is returned.
    fn to_u128(&self) -> Option<u128> {
        match self.to_u64() { Some(u) => Some(u as u128), None => None }
    }

    /// Converts the value of `self` to an `f32`. Overflows may map to positive
    /// or negative infinity, otherwise `None` is returned if the value cannot
    /// be represented by an `f32`.
    fn to_f32(&self) -> Option<f32> {
        match self.to_f64() {
            Some(v) => if v.is_finite() && (v < f32::MIN as f64 || v > f32::MAX as f64) { None } else { Some(v as f32) },
            None => None,
        }
    }

    /// Converts the value of `self` to an `f64`. Overflows may map to positive
    /// or negative infinity, otherwise `None` is returned if the value cannot
    /// be represented by an `f64`.
    fn to_f64(&self) -> Option<f64> {
        match self.to_i64() {
            Some(i) => Some(i as f64),
            None => match self.to_u64() { Some(u) => Some(u as f64), None => None },
        }
    }
}
}

macro_rules! impl_to_primitive_int_to_int {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            let min = $DstT::MIN as $SrcT;
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() <= size_of::<$DstT>() || (min <= *self && *self <= max) {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_int_to_uint {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if 0 <= *self && (size_of::<$SrcT>() <= size_of::<$DstT>() || *self <= max) {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_int {
    ($T:ident) => {
        c0nst::c0nst! {
        c0nst impl ToPrimitive for $T {
            impl_to_primitive_int_to_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
                fn to_i128 -> i128;
            }

            impl_to_primitive_int_to_uint! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
                fn to_u128 -> u128;
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                Some(*self as f32)
            }
            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(*self as f64)
            }
        }
        }
    };
}

impl_to_primitive_int!(isize);
impl_to_primitive_int!(i8);
impl_to_primitive_int!(i16);
impl_to_primitive_int!(i32);
impl_to_primitive_int!(i64);
impl_to_primitive_int!(i128);

macro_rules! impl_to_primitive_uint_to_int {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() < size_of::<$DstT>() || *self <= max {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_uint_to_uint {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() <= size_of::<$DstT>() || *self <= max {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_uint {
    ($T:ident) => {
        c0nst::c0nst! {
        c0nst impl ToPrimitive for $T {
            impl_to_primitive_uint_to_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
                fn to_i128 -> i128;
            }

            impl_to_primitive_uint_to_uint! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
                fn to_u128 -> u128;
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                Some(*self as f32)
            }
            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(*self as f64)
            }
        }
        }
    };
}

impl_to_primitive_uint!(usize);
impl_to_primitive_uint!(u8);
impl_to_primitive_uint!(u16);
impl_to_primitive_uint!(u32);
impl_to_primitive_uint!(u64);
impl_to_primitive_uint!(u128);

macro_rules! impl_to_primitive_nonzero_to_method {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            self.get().$method()
        }
    )*}
}

macro_rules! impl_to_primitive_nonzero {
    ($T:ident) => {
        c0nst::c0nst! {
        c0nst impl ToPrimitive for $T {
            impl_to_primitive_nonzero_to_method! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
                fn to_i128 -> i128;

                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
                fn to_u128 -> u128;

                fn to_f32 -> f32;
                fn to_f64 -> f64;
            }
        }
        }
    };
}

impl_to_primitive_nonzero!(NonZeroUsize);
impl_to_primitive_nonzero!(NonZeroU8);
impl_to_primitive_nonzero!(NonZeroU16);
impl_to_primitive_nonzero!(NonZeroU32);
impl_to_primitive_nonzero!(NonZeroU64);
impl_to_primitive_nonzero!(NonZeroU128);

impl_to_primitive_nonzero!(NonZeroIsize);
impl_to_primitive_nonzero!(NonZeroI8);
impl_to_primitive_nonzero!(NonZeroI16);
impl_to_primitive_nonzero!(NonZeroI32);
impl_to_primitive_nonzero!(NonZeroI64);
impl_to_primitive_nonzero!(NonZeroI128);

macro_rules! impl_to_primitive_float_to_float {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            // We can safely cast all values, whether NaN, +-inf, or finite.
            // Finite values that are reducing size may saturate to +-inf.
            Some(*self as $DstT)
        }
    )*}
}

macro_rules! float_to_int_unchecked {
    // The range check above already guarantees the value is representable.
    // `to_int_unchecked` would be slightly faster but isn't const yet; the
    // saturating `as` cast gives the same answer for in-range inputs.
    ($float:expr => $int:ty) => {
        $float as $int
    };
}

macro_rules! impl_to_primitive_float_to_signed_int {
    ($f:ident : $( fn $method:ident -> $i:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$i> {
            // Float as int truncates toward zero, so we want to allow values
            // in the exclusive range `(MIN-1, MAX+1)`.
            if size_of::<$f>() > size_of::<$i>() {
                // With a larger size, we can represent the range exactly.
                const MIN_M1: $f = $i::MIN as $f - 1.0;
                const MAX_P1: $f = $i::MAX as $f + 1.0;
                if *self > MIN_M1 && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $i));
                }
            } else {
                // We can't represent `MIN-1` exactly, but there's no fractional part
                // at this magnitude, so we can just use a `MIN` inclusive boundary.
                const MIN: $f = $i::MIN as $f;
                // We can't represent `MAX` exactly, but it will round up to exactly
                // `MAX+1` (a power of two) when we cast it.
                const MAX_P1: $f = $i::MAX as $f;
                if *self >= MIN && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $i));
                }
            }
            None
        }
    )*}
}

macro_rules! impl_to_primitive_float_to_unsigned_int {
    ($f:ident : $( fn $method:ident -> $u:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$u> {
            // Float as int truncates toward zero, so we want to allow values
            // in the exclusive range `(-1, MAX+1)`.
            if size_of::<$f>() > size_of::<$u>() {
                // With a larger size, we can represent the range exactly.
                const MAX_P1: $f = $u::MAX as $f + 1.0;
                if *self > -1.0 && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $u));
                }
            } else {
                // We can't represent `MAX` exactly, but it will round up to exactly
                // `MAX+1` (a power of two) when we cast it.
                // (`u128::MAX as f32` is infinity, but this is still ok.)
                const MAX_P1: $f = $u::MAX as $f;
                if *self > -1.0 && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $u));
                }
            }
            None
        }
    )*}
}

macro_rules! impl_to_primitive_float {
    ($T:ident) => {
        c0nst::c0nst! {
        c0nst impl ToPrimitive for $T {
            impl_to_primitive_float_to_signed_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
                fn to_i128 -> i128;
            }

            impl_to_primitive_float_to_unsigned_int! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
                fn to_u128 -> u128;
            }

            impl_to_primitive_float_to_float! { $T:
                fn to_f32 -> f32;
                fn to_f64 -> f64;
            }
        }
        }
    };
}

impl_to_primitive_float!(f32);
impl_to_primitive_float!(f64);

c0nst::c0nst! {
/// A generic trait for converting a number to a value.
///
/// A value can be represented by the target type when it lies within
/// the range of scalars supported by the target type.
/// For example, a negative integer cannot be represented by an unsigned
/// integer type, and an `i64` with a very high magnitude might not be
/// convertible to an `i32`.
/// On the other hand, conversions with possible precision loss or truncation
/// are admitted, like an `f32` with a decimal part to an integer type, or
/// even a large `f64` saturating to `f32` infinity.
pub c0nst trait FromPrimitive: Sized {
    /// Converts an `isize` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_isize(n: isize) -> Option<Self> { Self::from_i64(n as i64) }

    /// Converts an `i8` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_i8(n: i8) -> Option<Self> { Self::from_i64(n as i64) }

    /// Converts an `i16` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_i16(n: i16) -> Option<Self> { Self::from_i64(n as i64) }

    /// Converts an `i32` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_i32(n: i32) -> Option<Self> { Self::from_i64(n as i64) }

    /// Converts an `i64` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_i64(n: i64) -> Option<Self>;

    /// Converts an `i128` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_i128(n: i128) -> Option<Self> {
        // Route non-negative values through `from_u128` so the full
        // `(i64::MAX, u64::MAX]` range stays reachable; negatives go through
        // `from_i64`.
        if n >= 0 {
            Self::from_u128(n as u128)
        } else if n >= i64::MIN as i128 {
            Self::from_i64(n as i64)
        } else {
            None
        }
    }

    /// Converts a `usize` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_usize(n: usize) -> Option<Self> { Self::from_u64(n as u64) }

    /// Converts an `u8` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_u8(n: u8) -> Option<Self> { Self::from_u64(n as u64) }

    /// Converts an `u16` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_u16(n: u16) -> Option<Self> { Self::from_u64(n as u64) }

    /// Converts an `u32` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_u32(n: u32) -> Option<Self> { Self::from_u64(n as u64) }

    /// Converts an `u64` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_u64(n: u64) -> Option<Self>;

    /// Converts an `u128` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_u128(n: u128) -> Option<Self> {
        if n <= u64::MAX as u128 { Self::from_u64(n as u64) } else { None }
    }

    /// Converts a `f32` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_f32(n: f32) -> Option<Self> { Self::from_f64(n as f64) }

    /// Converts a `f64` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_f64(n: f64) -> Option<Self> {
        match crate::ToPrimitive::to_i64(&n) {
            Some(i) => Self::from_i64(i),
            None => match crate::ToPrimitive::to_u64(&n) { Some(u) => Self::from_u64(u), None => None },
        }
    }
}
}

macro_rules! impl_from_primitive {
    ($T:ty, $to_ty:ident) => {
        c0nst::c0nst! {
        c0nst impl FromPrimitive for $T {
            #[inline]
            fn from_isize(n: isize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i8(n: i8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i16(n: i16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i32(n: i32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i64(n: i64) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i128(n: i128) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_usize(n: usize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u8(n: u8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u16(n: u16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u32(n: u32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u64(n: u64) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u128(n: u128) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_f32(n: f32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_f64(n: f64) -> Option<$T> {
                n.$to_ty()
            }
        }
        }
    };
}

impl_from_primitive!(isize, to_isize);
impl_from_primitive!(i8, to_i8);
impl_from_primitive!(i16, to_i16);
impl_from_primitive!(i32, to_i32);
impl_from_primitive!(i64, to_i64);
impl_from_primitive!(i128, to_i128);
impl_from_primitive!(usize, to_usize);
impl_from_primitive!(u8, to_u8);
impl_from_primitive!(u16, to_u16);
impl_from_primitive!(u32, to_u32);
impl_from_primitive!(u64, to_u64);
impl_from_primitive!(u128, to_u128);
impl_from_primitive!(f32, to_f32);
impl_from_primitive!(f64, to_f64);

// `Option::and_then` is not yet a const fn; written as `match` to stay
// const-friendly while preserving semantics.
macro_rules! impl_from_primitive_nonzero_one {
    ($t:ty, $T:ty, $to_ty:ident, $method:ident) => {
        #[inline]
        fn $method(n: $t) -> Option<$T> {
            match n.$to_ty() {
                Some(v) => Self::new(v),
                None => None,
            }
        }
    };
}

macro_rules! impl_from_primitive_nonzero {
    ($T:ty, $to_ty:ident) => {
        c0nst::c0nst! {
        c0nst impl FromPrimitive for $T {
            impl_from_primitive_nonzero_one!(isize, $T, $to_ty, from_isize);
            impl_from_primitive_nonzero_one!(i8,    $T, $to_ty, from_i8);
            impl_from_primitive_nonzero_one!(i16,   $T, $to_ty, from_i16);
            impl_from_primitive_nonzero_one!(i32,   $T, $to_ty, from_i32);
            impl_from_primitive_nonzero_one!(i64,   $T, $to_ty, from_i64);
            impl_from_primitive_nonzero_one!(i128,  $T, $to_ty, from_i128);

            impl_from_primitive_nonzero_one!(usize, $T, $to_ty, from_usize);
            impl_from_primitive_nonzero_one!(u8,    $T, $to_ty, from_u8);
            impl_from_primitive_nonzero_one!(u16,   $T, $to_ty, from_u16);
            impl_from_primitive_nonzero_one!(u32,   $T, $to_ty, from_u32);
            impl_from_primitive_nonzero_one!(u64,   $T, $to_ty, from_u64);
            impl_from_primitive_nonzero_one!(u128,  $T, $to_ty, from_u128);

            impl_from_primitive_nonzero_one!(f32, $T, $to_ty, from_f32);
            impl_from_primitive_nonzero_one!(f64, $T, $to_ty, from_f64);
        }
        }
    };
}

impl_from_primitive_nonzero!(NonZeroIsize, to_isize);
impl_from_primitive_nonzero!(NonZeroI8, to_i8);
impl_from_primitive_nonzero!(NonZeroI16, to_i16);
impl_from_primitive_nonzero!(NonZeroI32, to_i32);
impl_from_primitive_nonzero!(NonZeroI64, to_i64);
impl_from_primitive_nonzero!(NonZeroI128, to_i128);
impl_from_primitive_nonzero!(NonZeroUsize, to_usize);
impl_from_primitive_nonzero!(NonZeroU8, to_u8);
impl_from_primitive_nonzero!(NonZeroU16, to_u16);
impl_from_primitive_nonzero!(NonZeroU32, to_u32);
impl_from_primitive_nonzero!(NonZeroU64, to_u64);
impl_from_primitive_nonzero!(NonZeroU128, to_u128);

macro_rules! impl_to_primitive_wrapping {
    ($( fn $method:ident -> $i:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$i> {
            (self.0).$method()
        }
    )*}
}

c0nst::c0nst! {
c0nst impl<T: [c0nst] ToPrimitive> ToPrimitive for Wrapping<T> {
    impl_to_primitive_wrapping! {
        fn to_isize -> isize;
        fn to_i8 -> i8;
        fn to_i16 -> i16;
        fn to_i32 -> i32;
        fn to_i64 -> i64;
        fn to_i128 -> i128;

        fn to_usize -> usize;
        fn to_u8 -> u8;
        fn to_u16 -> u16;
        fn to_u32 -> u32;
        fn to_u64 -> u64;
        fn to_u128 -> u128;

        fn to_f32 -> f32;
        fn to_f64 -> f64;
    }
}
}

macro_rules! impl_from_primitive_wrapping {
    ($( fn $method:ident ( $i:ident ); )*) => {$(
        #[inline]
        fn $method(n: $i) -> Option<Self> {
            // Hand-rolled match — `Option::map` is not yet a const fn.
            match T::$method(n) {
                Some(v) => Some(Wrapping(v)),
                None => None,
            }
        }
    )*}
}

c0nst::c0nst! {
c0nst impl<T: [c0nst] FromPrimitive + [c0nst] Destruct> FromPrimitive for Wrapping<T> {
    impl_from_primitive_wrapping! {
        fn from_isize(isize);
        fn from_i8(i8);
        fn from_i16(i16);
        fn from_i32(i32);
        fn from_i64(i64);
        fn from_i128(i128);

        fn from_usize(usize);
        fn from_u8(u8);
        fn from_u16(u16);
        fn from_u32(u32);
        fn from_u64(u64);
        fn from_u128(u128);

        fn from_f32(f32);
        fn from_f64(f64);
    }
}
}

c0nst::c0nst! {
/// Cast from one machine scalar to another.
///
/// # Examples
///
/// ```
/// # use const_num_traits as num;
/// let twenty: f32 = num::cast(0x14).unwrap();
/// assert_eq!(twenty, 20f32);
/// ```
///
#[inline]
pub c0nst fn cast<T: [c0nst] NumCast + [c0nst] Destruct, U: [c0nst] NumCast>(n: T) -> Option<U> {
    NumCast::from(n)
}
}

c0nst::c0nst! {
/// An interface for casting between machine scalars.
pub c0nst trait NumCast: Sized + [c0nst] ToPrimitive {
    /// Creates a number from another value that can be converted into
    /// a primitive via the `ToPrimitive` trait. If the source value cannot be
    /// represented by the target type, then `None` is returned.
    ///
    /// A value can be represented by the target type when it lies within
    /// the range of scalars supported by the target type.
    /// For example, a negative integer cannot be represented by an unsigned
    /// integer type, and an `i64` with a very high magnitude might not be
    /// convertible to an `i32`.
    /// On the other hand, conversions with possible precision loss or truncation
    /// are admitted, like an `f32` with a decimal part to an integer type, or
    /// even a large `f64` saturating to `f32` infinity.
    fn from<T: [c0nst] ToPrimitive + [c0nst] Destruct>(n: T) -> Option<Self>;
}
}

macro_rules! impl_num_cast {
    ($T:ty, $conv:ident) => {
        c0nst::c0nst! {
        c0nst impl NumCast for $T {
            #[inline]
            fn from<N: [c0nst] ToPrimitive + [c0nst] Destruct>(n: N) -> Option<$T> {
                n.$conv()
            }
        }
        }
    };
}

impl_num_cast!(u8, to_u8);
impl_num_cast!(u16, to_u16);
impl_num_cast!(u32, to_u32);
impl_num_cast!(u64, to_u64);
impl_num_cast!(u128, to_u128);
impl_num_cast!(usize, to_usize);
impl_num_cast!(i8, to_i8);
impl_num_cast!(i16, to_i16);
impl_num_cast!(i32, to_i32);
impl_num_cast!(i64, to_i64);
impl_num_cast!(i128, to_i128);
impl_num_cast!(isize, to_isize);
impl_num_cast!(f32, to_f32);
impl_num_cast!(f64, to_f64);

macro_rules! impl_num_cast_nonzero {
    ($T:ty, $conv:ident) => {
        c0nst::c0nst! {
        c0nst impl NumCast for $T {
            #[inline]
            fn from<N: [c0nst] ToPrimitive + [c0nst] Destruct>(n: N) -> Option<$T> {
                // `Option::and_then` isn't a const fn yet — hand-roll as match.
                match n.$conv() {
                    Some(v) => Self::new(v),
                    None => None,
                }
            }
        }
        }
    };
}

impl_num_cast_nonzero!(NonZeroUsize, to_usize);
impl_num_cast_nonzero!(NonZeroU8, to_u8);
impl_num_cast_nonzero!(NonZeroU16, to_u16);
impl_num_cast_nonzero!(NonZeroU32, to_u32);
impl_num_cast_nonzero!(NonZeroU64, to_u64);
impl_num_cast_nonzero!(NonZeroU128, to_u128);

impl_num_cast_nonzero!(NonZeroIsize, to_isize);
impl_num_cast_nonzero!(NonZeroI8, to_i8);
impl_num_cast_nonzero!(NonZeroI16, to_i16);
impl_num_cast_nonzero!(NonZeroI32, to_i32);
impl_num_cast_nonzero!(NonZeroI64, to_i64);
impl_num_cast_nonzero!(NonZeroI128, to_i128);

c0nst::c0nst! {
c0nst impl<T: [c0nst] NumCast + [c0nst] Destruct> NumCast for Wrapping<T> {
    fn from<U: [c0nst] ToPrimitive + [c0nst] Destruct>(n: U) -> Option<Self> {
        // Hand-rolled match — `Option::map` is not yet a const fn.
        match T::from(n) {
            Some(v) => Some(Wrapping(v)),
            None => None,
        }
    }
}
}

c0nst::c0nst! {
/// A generic interface for casting between machine scalars with the
/// `as` operator, which admits narrowing and precision loss.
/// Implementers of this trait `AsPrimitive` should behave like a primitive
/// numeric type (e.g. a newtype around another primitive), and the
/// intended conversion must never fail.
///
/// # Examples
///
/// ```
/// # use const_num_traits::AsPrimitive;
/// let three: i32 = (3.14159265f32).as_();
/// assert_eq!(three, 3);
/// ```
///
/// # Safety
///
/// **In Rust versions before 1.45.0**, some uses of the `as` operator were not entirely safe.
/// In particular, it was undefined behavior if
/// a truncated floating point value could not fit in the target integer
/// type ([#10184](https://github.com/rust-lang/rust/issues/10184)).
///
/// ```ignore
/// # use const_num_traits::AsPrimitive;
/// let x: u8 = (1.04E+17).as_(); // UB
/// ```
///
pub c0nst trait AsPrimitive<T>: 'static + Copy
where
    T: 'static + Copy,
{
    /// Convert a value to another, using the `as` operator.
    fn as_(self) -> T;
}
}

macro_rules! impl_as_primitive {
    (@ $T: ty =>  impl $U: ty ) => {
        c0nst::c0nst! {
        c0nst impl AsPrimitive<$U> for $T {
            #[inline] fn as_(self) -> $U { self as $U }
        }
        }
    };
    (@ $T: ty => { $( $U: ty ),* } ) => {$(
        impl_as_primitive!(@ $T => impl $U);
    )*};
    ($T: ty => { $( $U: ty ),* } ) => {
        impl_as_primitive!(@ $T => { $( $U ),* });
        impl_as_primitive!(@ $T => { u8, u16, u32, u64, u128, usize });
        impl_as_primitive!(@ $T => { i8, i16, i32, i64, i128, isize });
    };
}

impl_as_primitive!(u8 => { char, f32, f64 });
impl_as_primitive!(i8 => { f32, f64 });
impl_as_primitive!(u16 => { f32, f64 });
impl_as_primitive!(i16 => { f32, f64 });
impl_as_primitive!(u32 => { f32, f64 });
impl_as_primitive!(i32 => { f32, f64 });
impl_as_primitive!(u64 => { f32, f64 });
impl_as_primitive!(i64 => { f32, f64 });
impl_as_primitive!(u128 => { f32, f64 });
impl_as_primitive!(i128 => { f32, f64 });
impl_as_primitive!(usize => { f32, f64 });
impl_as_primitive!(isize => { f32, f64 });
impl_as_primitive!(f32 => { f32, f64 });
impl_as_primitive!(f64 => { f32, f64 });
impl_as_primitive!(char => { char });
impl_as_primitive!(bool => {});

/// Implements [`ToPrimitive`] from just a `to_i64` and a `to_u64`
/// conversion, deriving the other twelve methods the way the original
/// num-traits default methods did.
///
/// The const-trait port had to strip the `ToPrimitive` defaults (their
/// bodies aren't const-portable), which broke the upstream "minimal impl"
/// pattern of overriding only `to_i64`/`to_u64`. This macro restores that
/// pattern for downstream implementors; the generated impl is a plain
/// (non-const) impl of the const trait, which is exactly what a stable
/// downstream crate gets anyway.
///
/// Caveats inherited from the upstream defaults: `to_i128`/`to_u128` route
/// through the 64-bit methods (types with real 128-bit range should
/// implement `ToPrimitive` by hand), and `to_f64` is exact only for values
/// representable in 64 bits.
///
/// ```
/// use const_num_traits::ToPrimitive;
///
/// struct MyInt(i32);
///
/// const_num_traits::impl_to_primitive_minimal! {
///     impl ToPrimitive for MyInt {
///         to_i64 = |s: &MyInt| Some(s.0 as i64),
///         to_u64 = |s: &MyInt| if s.0 >= 0 { Some(s.0 as u64) } else { None },
///     }
/// }
///
/// assert_eq!(MyInt(-5).to_i8(), Some(-5i8));
/// assert_eq!(MyInt(-5).to_u32(), None);
/// assert_eq!(MyInt(300).to_u8(), None);
/// assert_eq!(MyInt(300).to_f64(), Some(300.0));
/// ```
#[macro_export]
macro_rules! impl_to_primitive_minimal {
    (impl ToPrimitive for $t:ty {
        to_i64 = $to_i64:expr,
        to_u64 = $to_u64:expr $(,)?
    }) => {
        impl $crate::ToPrimitive for $t {
            #[inline]
            fn to_i64(&self) -> Option<i64> {
                ($to_i64)(self)
            }

            #[inline]
            fn to_u64(&self) -> Option<u64> {
                ($to_u64)(self)
            }

            #[inline]
            fn to_isize(&self) -> Option<isize> {
                self.to_i64()
                    .and_then(|v| $crate::CheckedCast::<isize>::checked_cast(v))
            }

            #[inline]
            fn to_i8(&self) -> Option<i8> {
                self.to_i64()
                    .and_then(|v| $crate::CheckedCast::<i8>::checked_cast(v))
            }

            #[inline]
            fn to_i16(&self) -> Option<i16> {
                self.to_i64()
                    .and_then(|v| $crate::CheckedCast::<i16>::checked_cast(v))
            }

            #[inline]
            fn to_i32(&self) -> Option<i32> {
                self.to_i64()
                    .and_then(|v| $crate::CheckedCast::<i32>::checked_cast(v))
            }

            #[inline]
            fn to_i128(&self) -> Option<i128> {
                match self.to_i64() {
                    Some(i) => Some(i128::from(i)),
                    None => self.to_u64().map(i128::from),
                }
            }

            #[inline]
            fn to_usize(&self) -> Option<usize> {
                self.to_u64()
                    .and_then(|v| $crate::CheckedCast::<usize>::checked_cast(v))
            }

            #[inline]
            fn to_u8(&self) -> Option<u8> {
                self.to_u64()
                    .and_then(|v| $crate::CheckedCast::<u8>::checked_cast(v))
            }

            #[inline]
            fn to_u16(&self) -> Option<u16> {
                self.to_u64()
                    .and_then(|v| $crate::CheckedCast::<u16>::checked_cast(v))
            }

            #[inline]
            fn to_u32(&self) -> Option<u32> {
                self.to_u64()
                    .and_then(|v| $crate::CheckedCast::<u32>::checked_cast(v))
            }

            #[inline]
            fn to_u128(&self) -> Option<u128> {
                self.to_u64().map(u128::from)
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                match self.to_f64() {
                    Some(v) => {
                        if v.is_finite() && (v < f32::MIN as f64 || v > f32::MAX as f64) {
                            None
                        } else {
                            Some(v as f32)
                        }
                    }
                    None => None,
                }
            }

            #[inline]
            fn to_f64(&self) -> Option<f64> {
                match self.to_i64() {
                    Some(i) => Some(i as f64),
                    None => self.to_u64().map(|u| u as f64),
                }
            }
        }
    };
}

/// Implements [`FromPrimitive`] from just a `from_i64` and a `from_u64`
/// conversion, deriving the other twelve methods the way the original
/// num-traits default methods did.
///
/// See [`impl_to_primitive_minimal!`] for why this exists and the caveats
/// (`from_i128`/`from_u128` route through 64 bits; the generated impl is a
/// plain, non-const impl of the const trait).
///
/// ```
/// use const_num_traits::FromPrimitive;
///
/// #[derive(Debug, PartialEq)]
/// struct MyInt(i32);
///
/// const_num_traits::impl_from_primitive_minimal! {
///     impl FromPrimitive for MyInt {
///         from_i64 = |n: i64| i32::try_from(n).ok().map(MyInt),
///         from_u64 = |n: u64| i32::try_from(n).ok().map(MyInt),
///     }
/// }
///
/// assert_eq!(MyInt::from_i8(-5), Some(MyInt(-5)));
/// assert_eq!(MyInt::from_u64(u64::MAX), None);
/// assert_eq!(MyInt::from_f64(300.7), Some(MyInt(300)));
/// ```
#[macro_export]
macro_rules! impl_from_primitive_minimal {
    (impl FromPrimitive for $t:ty {
        from_i64 = $from_i64:expr,
        from_u64 = $from_u64:expr $(,)?
    }) => {
        impl $crate::FromPrimitive for $t {
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                ($from_i64)(n)
            }

            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                ($from_u64)(n)
            }

            #[inline]
            fn from_isize(n: isize) -> Option<Self> {
                Self::from_i64(n as i64)
            }

            #[inline]
            fn from_i8(n: i8) -> Option<Self> {
                Self::from_i64(i64::from(n))
            }

            #[inline]
            fn from_i16(n: i16) -> Option<Self> {
                Self::from_i64(i64::from(n))
            }

            #[inline]
            fn from_i32(n: i32) -> Option<Self> {
                Self::from_i64(i64::from(n))
            }

            #[inline]
            fn from_i128(n: i128) -> Option<Self> {
                if n >= 0 {
                    u64::try_from(n).ok().and_then(Self::from_u64)
                } else {
                    i64::try_from(n).ok().and_then(Self::from_i64)
                }
            }

            #[inline]
            fn from_usize(n: usize) -> Option<Self> {
                Self::from_u64(n as u64)
            }

            #[inline]
            fn from_u8(n: u8) -> Option<Self> {
                Self::from_u64(u64::from(n))
            }

            #[inline]
            fn from_u16(n: u16) -> Option<Self> {
                Self::from_u64(u64::from(n))
            }

            #[inline]
            fn from_u32(n: u32) -> Option<Self> {
                Self::from_u64(u64::from(n))
            }

            #[inline]
            fn from_u128(n: u128) -> Option<Self> {
                u64::try_from(n).ok().and_then(Self::from_u64)
            }

            #[inline]
            fn from_f32(n: f32) -> Option<Self> {
                Self::from_f64(f64::from(n))
            }

            #[inline]
            fn from_f64(n: f64) -> Option<Self> {
                match $crate::ToPrimitive::to_i64(&n) {
                    Some(i) => Self::from_i64(i),
                    None => $crate::ToPrimitive::to_u64(&n).and_then(Self::from_u64),
                }
            }
        }
    };
}

#[cfg(test)]
mod default_tests {
    use crate::{FromPrimitive, ToPrimitive};

    // The "minimal impl" pattern (override only the i64/u64 fundamentals,
    // inherit the other 12 via the restored defaults) — float8's shape.
    #[derive(Debug, PartialEq)]
    struct N(i32);

    impl ToPrimitive for N {
        fn to_i64(&self) -> Option<i64> {
            Some(self.0 as i64)
        }
        fn to_u64(&self) -> Option<u64> {
            if self.0 >= 0 {
                Some(self.0 as u64)
            } else {
                None
            }
        }
    }
    impl FromPrimitive for N {
        fn from_i64(n: i64) -> Option<Self> {
            if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
                Some(N(n as i32))
            } else {
                None
            }
        }
        fn from_u64(n: u64) -> Option<Self> {
            if n <= i32::MAX as u64 {
                Some(N(n as i32))
            } else {
                None
            }
        }
    }

    #[test]
    fn minimal_impl_inherits_defaults() {
        // derived to_* defaults
        assert_eq!(N(-5).to_i8(), Some(-5i8));
        assert_eq!(N(-5).to_u32(), None);
        assert_eq!(N(300).to_u8(), None);
        assert_eq!(N(300).to_i128(), Some(300i128));
        assert_eq!(N(300).to_f64(), Some(300.0));
        assert_eq!(N(300).to_f32(), Some(300.0));
        // derived from_* defaults
        assert_eq!(N::from_i8(-5), Some(N(-5)));
        assert_eq!(N::from_u128(u128::MAX), None);
        assert_eq!(N::from_i128(-7), Some(N(-7)));
        assert_eq!(N::from_f64(300.7), Some(N(300)));
        assert_eq!(N::from_f32(42.0), Some(N(42)));
    }

    // A `u64`-backed minimal type, to exercise the `(i64::MAX, u64::MAX]` range
    // that `N` (i32-backed) can't reach. Manual impl inherits the trait defaults.
    #[derive(Debug, PartialEq)]
    struct U(u64);
    impl ToPrimitive for U {
        fn to_i64(&self) -> Option<i64> {
            if self.0 <= i64::MAX as u64 {
                Some(self.0 as i64)
            } else {
                None
            }
        }
        fn to_u64(&self) -> Option<u64> {
            Some(self.0)
        }
    }
    impl FromPrimitive for U {
        fn from_i64(n: i64) -> Option<Self> {
            if n >= 0 { Some(U(n as u64)) } else { None }
        }
        fn from_u64(n: u64) -> Option<Self> {
            Some(U(n))
        }
    }

    // Same shape, but generated by the minimal *macros* (covers their i128 paths).
    #[derive(Debug, PartialEq)]
    struct UM(u64);
    crate::impl_to_primitive_minimal! {
        impl ToPrimitive for UM {
            to_i64 = |s: &UM| if s.0 <= i64::MAX as u64 { Some(s.0 as i64) } else { None },
            to_u64 = |s: &UM| Some(s.0),
        }
    }
    crate::impl_from_primitive_minimal! {
        impl FromPrimitive for UM {
            from_i64 = |n: i64| if n >= 0 { Some(UM(n as u64)) } else { None },
            from_u64 = |n: u64| Some(UM(n)),
        }
    }

    #[test]
    fn i128_conversions_cover_high_u64_range() {
        let big = 1u64 << 63; // > i64::MAX, but fits u64 and i128 exactly
        let big_i = big as i128;

        // trait defaults (U): to_i128 falls back to to_u64; from_i128 routes
        // non-negative through from_u64. Both were `None` before the fix.
        assert_eq!(U(big).to_i128(), Some(big_i));
        assert_eq!(U::from_i128(big_i), Some(U(big)));
        // minimal macros (UM): same paths
        assert_eq!(UM(big).to_i128(), Some(big_i));
        assert_eq!(UM::from_i128(big_i), Some(UM(big)));

        // ordinary ranges and rejections still hold
        assert_eq!(U(5).to_i128(), Some(5));
        assert_eq!(U::from_i128(-1), None); // unsigned-backed
        assert_eq!(UM::from_i128(u64::MAX as i128 + 1), None); // exceeds u64
    }
}
