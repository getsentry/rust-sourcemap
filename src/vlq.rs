//! Implements utilities for dealing with the sourcemap vlq encoding.

use std::fmt::{self, Write};

use fallible_iterator::FallibleIterator;

use crate::errors::{Error, Result};

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const B64: [i8; 128] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1, -1, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, -1, -1, -1, -1, -1,
];

/// A BASE64 VLQ (variable-length quantity) number parsed from a sourcemap.
///
/// These are defined in <https://tc39.es/ecma426/#sec-base64-vlq> in the
/// sourcemap spec.
///
/// This type can be encoded back into a BASE64 string representation
/// via its `Display` impl.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vlq(u32);

impl Vlq {
    /// Interprets the VLQ as an unsigned 32-bit number.
    pub fn unsigned(&self) -> u32 {
        self.0
    }

    /// Interprets the VLQ as a signed 32-bit number.
    ///
    /// NB: This is _not_ equivalent to `Self::unsigned() as i32`.
    /// See <https://tc39.es/ecma426/#sec-VLQSignedValue> for how
    /// an unsigned value is converted to a signed one.
    pub fn signed(&self) -> i32 {
        let neg = (self.0 & 1) != 0;
        let out = (self.0 >> 1) as i32;
        if neg && out == 0 {
            i32::MIN
        } else if neg {
            -out
        } else {
            out
        }
    }

    /// Constructs a `Self` from a signed 32-bit number.
    pub fn from_signed(value: i32) -> Self {
        if value == i32::MIN {
            Self(1)
        } else if value < 0 {
            Self((((-value) << 1) + 1) as u32)
        } else {
            Self((value << 1) as u32)
        }
    }

    /// Decodes one `Self` from the front of a string slice,
    /// returning the decoded VLQ and the rest of the string
    /// slice if successful.
    pub fn decode(segment: &str) -> Result<(Self, &str)> {
        if segment.is_empty() {
            return Err(Error::VlqNoValues);
        }
        let mut cur = 0;
        let mut shift = 0;

        let mut chars = segment.chars();
        for c in &mut chars {
            if !c.is_ascii() {
                return Err(Error::InvalidBase64(c));
            }
            let enc = B64[c as usize];
            if enc < 0 {
                return Err(Error::InvalidBase64(c));
            }
            let val = (enc & 0b11111) as u32;
            let finished = (enc >> 5) == 0;
            cur += val.checked_shl(shift).ok_or(Error::VlqOverflow)?;
            shift += 5;

            if finished {
                return Ok((Self(cur), chars.as_str()));
            }
        }

        // We're out of bytes, but the continuation bit is set.
        // This means a malformed VLQ.
        Err(Error::VlqTruncated)
    }
}

impl fmt::Display for Vlq {
    /// Formats a VLQ as BASE64.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = self.0;
        loop {
            let mut digit = current & 0b11111;
            current >>= 5;

            if current > 0 {
                digit |= 1 << 5;
            }

            f.write_char(B64_CHARS[digit as usize] as char)?;

            if current == 0 {
                break;
            }
        }

        Ok(())
    }
}

/// The iterator returned by [`decode_vlqs`].
#[derive(Debug, Clone)]
pub struct DecodeVlqs<'a> {
    returned: u32,
    current: &'a str,
}

impl<'a> DecodeVlqs<'a> {
    /// Returns the next decoded VLQ.
    ///
    /// Unlike [`next`](Self::next), this returns
    /// an error if no VLQs are left in the segment. Use this in
    /// situations where the absence of the next VLQ is an error.
    pub fn next_required(&mut self) -> Result<Vlq> {
        self.next()?.ok_or(Error::BadSegmentSize(self.returned))
    }
}

impl FallibleIterator for DecodeVlqs<'_> {
    type Item = Vlq;
    type Error = Error;

    fn next(&mut self) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match Vlq::decode(self.current) {
            Ok((val, rest)) => {
                self.current = rest;
                self.returned += 1;
                Ok(Some(val))
            }
            Err(Error::VlqNoValues) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Returns a [`FallibleIterator`] over the VLQs encoded in
/// the provided string slice.
pub fn decode_vlqs(segment: &str) -> DecodeVlqs<'_> {
    DecodeVlqs {
        current: segment,
        returned: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_vlq_segment_signed(segment: &str) -> Result<Vec<i32>> {
        let mut out = Vec::new();
        let mut vlqs = decode_vlqs(segment);
        while let Some(vlq) = vlqs.next()? {
            out.push(vlq.signed());
        }

        Ok(out)
    }

    fn generate_vlq_segment_signed(nums: &[i32]) -> String {
        let mut rv = String::new();
        for &n in nums {
            write!(&mut rv, "{}", Vlq::from_signed(n)).unwrap();
        }

        rv
    }

    #[test]
    fn test_vlq_decode() {
        let rv = parse_vlq_segment_signed("AAAA").unwrap();
        assert_eq!(rv, [0, 0, 0, 0]);
        let rv = parse_vlq_segment_signed("GAAIA").unwrap();
        assert_eq!(rv, vec![3, 0, 0, 4, 0]);
        let rv = parse_vlq_segment_signed("B").unwrap();
        assert_eq!(rv, vec![i32::MIN]);
    }

    #[test]
    fn test_vlq_encode() {
        let rv = generate_vlq_segment_signed(&[0, 0, 0, 0]);
        assert_eq!(rv.as_str(), "AAAA");
        let rv = generate_vlq_segment_signed(&[3, 0, 0, 4, 0]);
        assert_eq!(rv.as_str(), "GAAIA");
        let rv = generate_vlq_segment_signed(&[i32::MIN]);
        assert_eq!(rv.as_str(), "B");
    }

    #[test]
    fn test_overflow() {
        match parse_vlq_segment_signed("00000000000000") {
            Err(Error::VlqOverflow) => {}
            e => {
                panic!("Unexpected result: {:?}", e);
            }
        }
    }
}
