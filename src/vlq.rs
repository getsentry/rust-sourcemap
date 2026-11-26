//! Implements utilities for dealing with the sourcemap vlq encoding.

use crate::errors::{Error, Result};

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const B64: [i8; 256] = [
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    62,
    -1,
    -1,
    -1,
    63,
    52,
    53,
    54,
    55,
    56,
    57,
    58,
    59,
    60,
    61,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
    24,
    25,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    26,
    27,
    28,
    29,
    30,
    31,
    32,
    33,
    34,
    35,
    36,
    37,
    38,
    39,
    40,
    41,
    42,
    43,
    44,
    45,
    46,
    47,
    48,
    49,
    50,
    51,
    -1,
    -1,
    -1,
    -1,
    -1 - 1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
    -1,
];

#[derive(Debug, Clone, Copy)]
pub struct Vlq(u32);

impl Vlq {
    pub fn unsigned(&self) -> u32 {
        self.0
    }

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

    pub fn from_unsigned(value: u32) -> Self {
        Self(value)
    }

    pub fn from_signed(value: i32) -> Self {
        if value == i32::MIN {
            Self(1)
        } else if value < 0 {
            Self((((-value) << 1) + 1) as u32)
        } else {
            Self((value << 1) as u32)
        }
    }

    pub fn encode(&self) -> VlqDigitIter {
        VlqDigitIter {
            current: self.0,
            finished: false,
        }
    }

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

#[derive(Debug, Clone)]
pub struct VlqDigitIter {
    current: u32,
    finished: bool,
}

impl Iterator for VlqDigitIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut digit = self.current & 0b11111;
        self.current >>= 5;

        if self.current > 0 {
            digit |= 1 << 5;
        } else {
            self.finished = true;
        }

        Some(B64_CHARS[digit as usize] as char)
    }
}

#[derive(Debug, Clone)]
pub struct DecodeVlqs<'a> {
    current: &'a str,
}

impl Iterator for DecodeVlqs<'_> {
    type Item = Result<Vlq>;

    fn next(&mut self) -> Option<Self::Item> {
        match Vlq::decode(self.current) {
            Ok((val, rest)) => {
                self.current = rest;
                Some(Ok(val))
            }
            Err(Error::VlqNoValues) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub fn decode_vlqs(segment: &str) -> DecodeVlqs<'_> {
    DecodeVlqs { current: segment }
}

/// Parses a VLQ segment into a vector.
pub fn parse_vlq_segment(segment: &str) -> Result<Vec<i64>> {
    let mut rv = vec![];

    parse_vlq_segment_into(segment, &mut rv)?;

    Ok(rv)
}

/// Parses a VLQ segment into a pre-allocated `Vec` instead of returning a new allocation.
pub(crate) fn parse_vlq_segment_into(segment: &str, rv: &mut Vec<i64>) -> Result<()> {
    for vlq in decode_vlqs(segment) {
        let vlq = vlq?;
        rv.push(vlq.signed() as i64);
    }
    Ok(())
}

/// Encodes a VLQ segment from a slice.
pub fn generate_vlq_segment(nums: &[i64]) -> Result<String> {
    let mut rv = String::new();
    for &num in nums {
        encode_vlq(&mut rv, num);
    }
    Ok(rv)
}

pub(crate) fn encode_vlq(out: &mut String, num: i64) {
    out.extend(Vlq::from_signed(num as i32).encode())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlq_decode() {
        let rv = parse_vlq_segment("AAAA").unwrap();
        assert_eq!(rv, vec![0, 0, 0, 0]);
        let rv = parse_vlq_segment("GAAIA").unwrap();
        assert_eq!(rv, vec![3, 0, 0, 4, 0]);
        let rv = parse_vlq_segment("B").unwrap();
        assert_eq!(rv, vec![i32::MIN as i64]);
    }

    #[test]
    fn test_vlq_encode() {
        let rv = generate_vlq_segment(&[0, 0, 0, 0]).unwrap();
        assert_eq!(rv.as_str(), "AAAA");
        let rv = generate_vlq_segment(&[3, 0, 0, 4, 0]).unwrap();
        assert_eq!(rv.as_str(), "GAAIA");
        let rv = generate_vlq_segment(&[i32::MIN as i64]).unwrap();
        assert_eq!(rv.as_str(), "B");
    }

    #[test]
    fn test_overflow() {
        match parse_vlq_segment("00000000000000") {
            Err(Error::VlqOverflow) => {}
            e => {
                panic!("Unexpected result: {:?}", e);
            }
        }
    }
}
