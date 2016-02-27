/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */


//! Alternative and extension to the unstable `char.encode_utf8()` and `char.encode_utf16()`.


// warnings
#![warn(missing_docs)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(len_without_is_empty))]// UtfxChar is never empty
#![cfg_attr(feature="clippy", allow(match_same_arms))]
// derive_hash_xor_eq: I think it works, but not tested yet.
// precedence: I prefer spaces to parentheses, but it's nice to recheck.

mod errors;
mod traits;
mod utf8_char;
mod utf8_iterator;
mod utf16_char;
mod utf16_iterator;

pub use traits::CharExt;
pub use utf8_char::Utf8Char;
pub use utf16_char::Utf16Char;
pub use utf8_iterator::Utf8Iterator;
pub use utf16_iterator::Utf16Iterator;
pub use traits::U8UtfExt;
pub use traits::U16UtfExt;

pub mod error {// keeping the public interface in one file
    //! Errors returned by various conversion methods in this crate.
    pub use utf8_char::FromStrError;
    pub use errors::{InvalidCodePoint};
    pub use errors::{InvalidUtf8FirstByte,InvalidUtf8};
    pub use errors::{InvalidUtf8Slice,InvalidUtf16Slice};
    pub use errors::{InvalidUtf8Array,InvalidUtf16Tuple};
}
