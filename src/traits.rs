/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

#![allow(unused_unsafe)]// explicit unsafe{} blocks in unsafe functions are a good thing.

use Utf8Char;
use Utf16Char;
use Utf8Iterator;
use Utf16Iterator;
use error::*;
extern crate std;
use std::{char,u32, mem};
use std::ops::Not;

// TODO better docs and tests

/// Methods for working with `u8`s UTF-8.
pub trait U8UtfExt {
    /// How many more bytes will you need to complete this codepoint?
    fn extra_utf8_bytes(self) -> Result<usize,InvalidUtf8FirstByte>;

    /// How many more bytes will you need to complete this codepoint?
    /// Assumes that self is a valid UTF-8 start.
    /// Returns `self.not().leading_zeros().saturating_sub(1)`
    fn extra_utf8_bytes_unchecked(self) -> usize;
}

impl U8UtfExt for u8 {
    /// Failures:
    ///
    /// * `128..192`: ContinuationByte
    /// * `240..`: TooLongSequence
    fn extra_utf8_bytes(self) -> Result<usize,InvalidUtf8FirstByte> {
        use error::InvalidUtf8FirstByte::{ContinuationByte,TooLongSeqence};
        match self.not().leading_zeros() {
            0           =>  Ok(0),// ascii
            1           =>  Err(ContinuationByte),// following byte
            n if n < 5  =>  Ok(n as usize-1),// start of multibyte
            _           =>  Err(TooLongSeqence),// too big
        }
    }
    fn extra_utf8_bytes_unchecked(self) -> usize {
        (self.not().leading_zeros()as usize).saturating_sub(1)
    }
}


/// Methods for working with `u16`s as UTF-16 units.
pub trait U16UtfExt {
    /// Will you need an extra unit to complete this codepoint?
    /// Returns `true` if it's a high surrogate (udc00...udfff),
    ///         `None` if it's a low surrogate (ud800...udbff),
    ///         or `false` if it's neither.
    fn utf16_needs_extra_unit(self) -> Option<bool>;
    /// Does this `u16` need another `u16` to complete a codepoint?
    fn utf16_is_leading_surrogate(self) -> bool;
}
impl U16UtfExt for u16 {
    /// # Failures:
    ///
    /// 0xdc00..0xe000
    fn utf16_needs_extra_unit(self) -> Option<bool> {match self {
        // https://en.wikipedia.org/wiki/UTF-16#U.2B10000_to_U.2B10FFFF
        0x_dc_00...0x_df_ff => None,
        0x_d8_00...0x_db_ff => Some(true),
        _                   => Some(false),
    }}

    /// Returns `(self & 0xfc00) == 0xd800`
    fn utf16_is_leading_surrogate(self) -> bool {
        (self & 0xfc00) == 0xd800// Clear the ten content bytes of a surrogate,
                                 // and see if it's a leading surrogate.
    }
}




/// Extension trait for `char` that adds methods for converting to and from UTF-8 or UTF-16.
pub trait CharExt: Sized {
    /// Get the UTF-8 representation of this codepoint.
    ///
    /// `Utf8Char` is to `[u8;4]` what `char` is to `u32`:
    /// a restricted type that cannot be mutated internally.
    fn to_utf8(self) -> Utf8Char;

    /// Get the UTF-16 representation of this codepoint.
    ///
    /// `Utf16Char` is to `(u16,Option<u16>)` what `char` is to `u32`:
    /// a restricted type that cannot be mutated internally.
    fn to_utf16(self) -> Utf16Char;

    /// Iterate over or [read](https://doc.rust-lang.org/std/io/trait.Read.html)
    /// the one to four bytes in the UTF-8 representation of this codepoint.
    fn iter_utf8_bytes(self) -> Utf8Iterator;

    /// Iterate over the one or two units in the UTF-16 representation of this codepoint.
    fn iter_utf16_units(self) -> Utf16Iterator;


    /// Convert this char to UTF-8, and then
    /// returns the number of bytes written.
    ///
    /// `None` is returned if the buffer is too small; then the buffer is left unmodified.
    /// A buffer of length four is always large enough.
    ///
    /// Similar to the unstable `.encode_utf8()`,
    /// but that method somehow still exist on stable, so I have to use a different name.
    fn to_utf8_slice(self,  dst: &mut[u8]) -> Option<usize>;

    /// Convert this char to UTF-16, and then
    /// returns the number of units written.
    ///
    /// `None` is returned if the buffer is too small; then the buffer is left unmodified.
    /// A buffer of length two is always large enough.
    ///
    /// Similar to the unstable `.encode_utf16()`,
    /// but that method somehow still exist on stable, so I have to use a different name.
    fn to_utf16_slice(self,  dst: &mut[u16]) -> Option<usize>;


    /// Convert this char to an UTF-8 array and lenght,
    /// The returned array is left-aligned, and the usize is how many bytes are used.
    /// The unused bytes are zero.
    fn to_utf8_array(self) -> ([u8; 4], usize);

    /// Convert this char to UTF-16.
    /// The second `u16` is `Some` if a surrogate pair is required.
    fn to_utf16_tuple(self) -> (u16, Option<u16>);



    /// Create a `char` from the start of a slice intepreted as UTF-8, and return how many bytes were needed.
    fn from_utf8_slice(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice>;

    /// Read one or two UTF-16 units into a `char`, and also return how many units were needed.
    fn from_utf16_slice(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice>;


    /// Convert an UTF-8 sequence as returned from `.to_utf8_array()` into a char
    fn from_utf8_array(utf8: [u8; 4]) -> Result<Self,InvalidUtf8Array>;

    /// Convert a UTF-16 pair as returned from `.to_utf16_tuple()` into a `char`.
    fn from_utf16_tuple(utf16: (u16, Option<u16>)) -> Result<Self, InvalidUtf16Tuple>;


    /// Convert an UTF-8 sequence into a char.
    /// The length of the slice is the length of the sequence, should be 1,2,3 or 4.
    ///
    /// # Panics:
    ///
    /// If the slice is empty
    unsafe fn from_utf8_exact_slice_unchecked(src: &[u8]) -> Self;

    /// Convert a UTF-16 pair as returned from `char.to_utf16_tuple()` into a `char`.
    unsafe fn from_utf16_tuple_unchecked(utf16: (u16, Option<u16>)) -> Self;


    /// Perform some extra validations compared to `char::from_u32_unchecked()`
    ///
    /// # Failures:
    ///
    /// * the value is greater than 0x10ffff
    /// * the value is between 0xd800 and 0xdfff (inclusive)
    fn from_u32_detailed(c: u32) -> Result<Self,InvalidCodePoint>;
}



impl CharExt for char {
      /////////
     //UTF-8//
    /////////

    fn to_utf8(self) -> Utf8Char {
        self.into()
    }
    fn iter_utf8_bytes(self) -> Utf8Iterator {
        self.to_utf8().into_iter()
    }
    fn to_utf8_slice(self,  dst: &mut[u8]) -> Option<usize> {
        self.to_utf8().to_slice(dst)
    }

    fn to_utf8_array(self) -> ([u8; 4], usize) {
        let len = self.len_utf8();
        let mut c = self as u32;
        if len == 1 {// ASCII, the common case
            ([c as u8, 0, 0, 0],  1)
        } else {
            let mut parts = 0;// convert to 6-bit bytes
                        parts |= c & 0x3f;  c>>=6;
            parts<<=8;  parts |= c & 0x3f;  c>>=6;
            parts<<=8;  parts |= c & 0x3f;  c>>=6;
            parts<<=8;  parts |= c & 0x3f;
            parts |= 0x80_80_80_80;// set the most significant bit
            parts >>= 8*(4-len);// right-align bytes
            // Now, unused bytes are zero, (which matters for Utf8Char.eq()) and the rest are 0b10xx_xxxx

            // set header on first byte
            parts |= (0xff_00u32 >> len)  &  0xff;// store length
            parts &= Not::not(1u32 << 7-len);// clear the next bit after it

            let bytes: [u8; 4] = unsafe{ mem::transmute(u32::from_le(parts)) };
            (bytes, len)
        }
    }


    /// Create a `char` from the start of a slice intepreted as UTF-8, and return how many bytes were needed.
    fn from_utf8_slice(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice> {
        use errors::InvalidUtf8::*;
        use errors::InvalidUtf8Slice::*;
        let first = *try!(src.first().ok_or(TooShort(1)));
        let extra = try!(first.extra_utf8_bytes().map_err(|e| Utf8(FirstByte(e)) ));
        if extra == 0 {
            return Ok((first as char, 1));
        } else if src.len() <= extra {
            return Err(TooShort(extra+1))
        }
        let src = &src[..1+extra];
        for (i, &b) in src.iter().enumerate().skip(1) {
            if b < 0b1000_0000  ||  b > 0b1011_1111 {
                return Err(Utf8(NotAContinuationByte(i)));
            }
        }
        if overlong(src[0], src[1]) {
            return Err(Utf8(OverLong));
        }
        let c = unsafe{ char::from_utf8_exact_slice_unchecked(src) };
        char::from_u32_detailed(c as u32).map(|c| (c,src.len()) ).map_err( CodePoint )
    }

    /// Convert an UTF-8 sequence as returned from `.to_utf8_array()` into a `char`
    fn from_utf8_array(utf8: [u8; 4]) -> Result<Self,InvalidUtf8Array> {
        use errors::InvalidUtf8::*;
        use errors::InvalidUtf8Array::*;
        let len = match utf8[0].extra_utf8_bytes() {
            Ok(0)     =>  return Ok(utf8[0] as char),
            Ok(l)     =>  l+1,
            Err(err)  =>  return Err(Utf8(FirstByte(err))),
        };
        for (i, &b) in utf8[..len].iter().enumerate().skip(1) {
            if b < 0b1000_0000  ||  b > 0b1011_1111 {
                return Err(Utf8(NotAContinuationByte(i)));
            }
        }

        if overlong(utf8[0], utf8[1]) {
            return Err(Utf8(OverLong));
        }
        let c = unsafe{ char::from_utf8_exact_slice_unchecked(&utf8[..len]) };
        char::from_u32_detailed(c as u32).map_err( CodePoint )
    }

    /// Convert an UTF-8 sequence into a char.
    /// The length of the slice is the length of the sequence, should be 1,2,3 or 4.
    ///
    /// # Panics:
    ///
    /// If the slice is empty
    unsafe fn from_utf8_exact_slice_unchecked(src: &[u8]) -> Self {
        if src.len() == 1 {
            src[0] as char
        } else {
            let mut c = src[0] as u32 & (0xff >> 2+src.len()-1);
            for b in &src[1..] {
                c = (c << 6)  |  (b & 0b00111111) as u32;
            }
            unsafe{ char::from_u32_unchecked(c) }
        }
    }



      //////////
     //UTF-16//
    //////////

    fn to_utf16(self) -> Utf16Char {
        Utf16Char::from(self)
    }
    fn iter_utf16_units(self) -> Utf16Iterator {
        self.to_utf16().into_iter()
    }

    fn to_utf16_slice(self,  dst: &mut[u16]) -> Option<usize> {
        let (first, second) = self.to_utf16_tuple();
        match (dst.len(), second) {
            (0, _)            =>  None,
            (1, Some(_))      =>  None,
            (_, Some(second)) => {dst[0] = first;
                                  dst[1] = second;
                                  Some(2)
                                 },
            (_, None)         => {dst[0] = first;
                                  Some(1)
                                 },
        }
    }

    /// Convert this char to UTF-16.
    /// The second `u16` is `Some` for surrogate pairs
    fn to_utf16_tuple(self) -> (u16, Option<u16>) {
        let c = self as u32;
        if c <= 0x_ff_ff {// single (or reserved, which we ignore)
            (c as u16, None)
        } else {// double (or too high, which we ignore)
            let c = c - 0x_01_00_00;
            let high = 0x_d8_00 + (c >> 10);
            let low = 0x_dc_00 + (c & 0x_03_ff);
            (high as u16,  Some(low as u16))
        }
    }


    /// Read one or two UTF-16 units into a `char`, and also return how many units were needed.
    fn from_utf16_slice(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice> {
        use errors::InvalidUtf16Slice::*;
        let first = *try!(src.first().ok_or(EmptySlice));
        match (first.utf16_needs_extra_unit(), src.get(1).cloned()) {
            (Some(false),              _            )  =>  Ok((1, None)),
            (Some(true) ,  Some(0x_dc_00...0x_df_ff))  =>  Ok((2, Some(src[1]))),
            (Some(true) ,  Some(         _         ))  =>  Err(SecondNotLowSurrogate),
            (Some(true) ,  None                     )  =>  Err(MissingSecond),
            (None       ,              _            )  =>  Err(FirstLowSurrogate),
        }.map(|(len,second)| (unsafe{ char::from_utf16_tuple_unchecked((first,second)) }, len) )
    }

    /// Convert a UTF-16 tuple as returned from `.to_utf16_tuple()` into a `char`.
    fn from_utf16_tuple(utf16: (u16, Option<u16>)) -> Result<Self, InvalidUtf16Tuple> {
        use errors::InvalidUtf16Tuple::*;
        match utf16 {
            (0x_00_00...0x_d7_ff, None) => Ok(()),// single
            (0x_e0_00...0x_ff_ff, None) => Ok(()),// single
            (0x_d8_00...0x_db_ff, Some(0x_dc_00...0x_df_ff)) => Ok(()),// correct surrogate pair
            (0x_d8_00...0x_db_ff, Some(_)) => Err(InvalidSecond),
            (0x_dc_00...0x_df_ff, None) => Err(MissingSecond),
            (0x_dc_00...0x_df_ff, _) => Err(FirstIsTrailingSurrogate),
            (_, Some(_)) => Err(SuperfluousSecond),// should be no second
            (_, _) => unreachable!()
        }.map(|_| unsafe{ char::from_utf16_tuple_unchecked(utf16) } )
    }

    /// Convert a UTF-16 tuple as returned from `char.to_utf16_tuple()` into a `char`.
    unsafe fn from_utf16_tuple_unchecked(utf16: (u16, Option<u16>)) -> Self {
        let mut c = utf16.0 as u32;
        if let Some(second) = utf16.1 {
            let high = (c-0x_d8_00) << 10;
            let low = second as u32 - 0x_dc_00;
            c = high | low;
            c += 0x_01_00_00;
        }
        unsafe{ char::from_u32_unchecked(c) }
    }


    fn from_u32_detailed(c: u32) -> Result<Self,InvalidCodePoint> {
        use errors::InvalidCodePoint::*;
        match c {
            // reserved for UTF-16 surrogate pairs
            0xd8_00...0xdf_ff => Err(Utf16Reserved),
            // too big
            0x11_00_00...u32::MAX => Err(TooHigh),
            _ => Ok(unsafe{ char::from_u32_unchecked(c) }),
        }
    }
}


// If all the data bits in the first byte are zero, the sequence might be longer than necessary
// When you go up one byte, you gain 6-1 data bits, so if the five first are zero it's too long.
// The first byte has 3 + (4-len) data bits, which we know are zero.
// The first two bits in the second byte are 10, which gets shifted out.
fn overlong(first: u8,  second: u8) -> bool {
    let both = ((first as u16) << 8)  |  (second << 2) as u16;
    let both = both << 1+both.not().leading_zeros();
    both.leading_zeros() >= 5
}
