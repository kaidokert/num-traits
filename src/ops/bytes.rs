use core::borrow::{Borrow, BorrowMut};
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use core::fmt::Debug;
use core::hash::Hash;
use core::mem::transmute;

pub trait ToFromBytes {
    type Bytes: Debug
        + AsRef<[u8]>
        + AsMut<[u8]>
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + Borrow<[u8]>
        + BorrowMut<[u8]>
        + Default;

    /// Return the memory representation of this number as a byte array in big-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::ToFromBytes;
    ///
    /// let bytes = 0x12345678u32.to_be_bytes();
    /// assert_eq!(bytes, [0x12, 0x34, 0x56, 0x78]);
    /// ```
    fn to_be_bytes(self) -> Self::Bytes;

    /// Return the memory representation of this number as a byte array in little-endian byte order.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::ToFromBytes;
    ///
    /// let bytes = 0x12345678u32.to_le_bytes();
    /// assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);
    /// ```
    fn to_le_bytes(self) -> Self::Bytes;

    /// Return the memory representation of this number as a byte array in native byte order.
    ///
    /// As the target platform's native endianness is used,
    /// portable code should use [`to_be_bytes`] or [`to_le_bytes`], as appropriate, instead.
    ///
    /// [`to_be_bytes`]: #method.to_be_bytes
    /// [`to_le_bytes`]: #method.to_le_bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::ToFromBytes;
    ///
    /// let bytes = 0x12345678u32.to_ne_bytes();
    /// assert_eq!(bytes, if cfg!(target_endian = "big") {
    ///     [0x12, 0x34, 0x56, 0x78]
    /// } else {
    ///     [0x78, 0x56, 0x34, 0x12]
    /// });
    /// ```
    fn to_ne_bytes(self) -> Self::Bytes;

    /// Create a number from its representation as a byte array in big endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::ToFromBytes;
    ///
    /// let value = u32::from_be_bytes([0x12, 0x34, 0x56, 0x78]);
    /// assert_eq!(value, 0x12345678);
    /// ```
    fn from_be_bytes(bytes: Self::Bytes) -> Self;

    /// Create a number from its representation as a byte array in little endian.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::ToFromBytes;
    ///
    /// let value = u32::from_le_bytes([0x78, 0x56, 0x34, 0x12]);
    /// assert_eq!(value, 0x12345678);
    /// ```
    fn from_le_bytes(bytes: Self::Bytes) -> Self;

    /// Create a number from its memory representation as a byte array in native endianness.
    ///
    /// As the target platform's native endianness is used,
    /// portable code likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as appropriate instead.
    ///
    /// [`from_be_bytes`]: #method.from_be_bytes
    /// [`from_le_bytes`]: #method.from_le_bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::ToFromBytes;
    ///
    /// let value = u32::from_ne_bytes(if cfg!(target_endian = "big") {
    ///     [0x12, 0x34, 0x56, 0x78]
    /// } else {
    ///     [0x78, 0x56, 0x34, 0x12]
    /// });
    /// assert_eq!(value, 0x12345678);
    /// ```
    fn from_ne_bytes(bytes: Self::Bytes) -> Self;
}

macro_rules! float_to_from_bytes_impl {
    ($T:ty, $I:ty, $L:expr) => {
        #[cfg(feature = "has_float_to_from_bytes")]
        impl ToFromBytes for $T {
            type Bytes = [u8; $L];

            #[inline]
            fn to_be_bytes(self) -> Self::Bytes {
                <$T>::to_be_bytes(self)
            }

            #[inline]
            fn to_le_bytes(self) -> Self::Bytes {
                <$T>::to_le_bytes(self)
            }

            #[inline]
            fn to_ne_bytes(self) -> Self::Bytes {
                <$T>::to_ne_bytes(self)
            }

            #[inline]
            fn from_be_bytes(bytes: Self::Bytes) -> Self {
                <$T>::from_be_bytes(bytes)
            }

            #[inline]
            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                <$T>::from_le_bytes(bytes)
            }

            #[inline]
            fn from_ne_bytes(bytes: Self::Bytes) -> Self {
                <$T>::from_ne_bytes(bytes)
            }
        }

        #[cfg(not(feature = "has_float_to_from_bytes"))]
        impl ToFromBytes for $T {
            type Bytes = [u8; $L];

            #[inline]
            fn to_be_bytes(self) -> Self::Bytes {
                <$I>::from_ne_bytes(self.to_ne_bytes()).to_be_bytes()
            }

            #[inline]
            fn to_le_bytes(self) -> Self::Bytes {
                <$I>::from_ne_bytes(self.to_ne_bytes()).to_le_bytes()
            }

            #[inline]
            fn to_ne_bytes(self) -> Self::Bytes {
                unsafe { transmute(self) }
            }

            #[inline]
            fn from_be_bytes(bytes: Self::Bytes) -> Self {
                Self::from_ne_bytes(<$I>::from_be_bytes(bytes).to_ne_bytes())
            }

            #[inline]
            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                Self::from_ne_bytes(<$I>::from_le_bytes(bytes).to_ne_bytes())
            }

            #[inline]
            fn from_ne_bytes(bytes: Self::Bytes) -> Self {
                unsafe { transmute(bytes) }
            }
        }
    };
}

macro_rules! int_to_from_bytes_impl {
    ($T:ty, $L:expr) => {
        #[cfg(feature = "has_int_to_from_bytes")]
        impl ToFromBytes for $T {
            type Bytes = [u8; $L];

            #[inline]
            fn to_be_bytes(self) -> Self::Bytes {
                <$T>::to_be_bytes(self)
            }

            #[inline]
            fn to_le_bytes(self) -> Self::Bytes {
                <$T>::to_le_bytes(self)
            }

            #[inline]
            fn to_ne_bytes(self) -> Self::Bytes {
                <$T>::to_ne_bytes(self)
            }

            #[inline]
            fn from_be_bytes(bytes: Self::Bytes) -> Self {
                <$T>::from_be_bytes(bytes)
            }

            #[inline]
            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                <$T>::from_le_bytes(bytes)
            }

            #[inline]
            fn from_ne_bytes(bytes: Self::Bytes) -> Self {
                <$T>::from_ne_bytes(bytes)
            }
        }

        #[cfg(not(feature = "has_int_to_from_bytes"))]
        impl ToFromBytes for $T {
            type Bytes = [u8; $L];

            #[inline]
            fn to_be_bytes(self) -> Self::Bytes {
                <$T>::to_ne_bytes(<$T>::to_be(self))
            }

            #[inline]
            fn to_le_bytes(self) -> Self::Bytes {
                <$T>::to_ne_bytes(<$T>::to_le(self))
            }

            #[inline]
            fn to_ne_bytes(self) -> Self::Bytes {
                unsafe { transmute(self) }
            }

            #[inline]
            fn from_be_bytes(bytes: Self::Bytes) -> Self {
                Self::from_be(Self::from_ne_bytes(bytes))
            }

            #[inline]
            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                Self::from_le(Self::from_ne_bytes(bytes))
            }

            #[inline]
            fn from_ne_bytes(bytes: Self::Bytes) -> Self {
                unsafe { transmute(bytes) }
            }
        }
    };
}

int_to_from_bytes_impl!(u8, 1);
int_to_from_bytes_impl!(u16, 2);
int_to_from_bytes_impl!(u32, 4);
int_to_from_bytes_impl!(u64, 8);
int_to_from_bytes_impl!(usize, 8);

int_to_from_bytes_impl!(i8, 1);
int_to_from_bytes_impl!(i16, 2);
int_to_from_bytes_impl!(i32, 4);
int_to_from_bytes_impl!(i64, 8);
int_to_from_bytes_impl!(isize, 8);

#[cfg(has_i128)]
int_to_from_bytes_impl!(u128, 16);
#[cfg(has_i128)]
int_to_from_bytes_impl!(i128, 16);

float_to_from_bytes_impl!(f32, u32, 4);
float_to_from_bytes_impl!(f64, u64, 8);
