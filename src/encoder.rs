use std::io::Write;

use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::view::BitView;
use serde_json::Value;

use crate::errors::Result;
use crate::jsontypes::{RawSection, RawSectionOffset, RawSourceMap};
use crate::types::{DecodedMap, SourceMap, SourceMapIndex};
use crate::vlq::encode_vlq;

pub trait Encodable {
    fn as_raw_sourcemap(&self) -> RawSourceMap;
}

pub fn encode<M: Encodable, W: Write>(sm: &M, mut w: W) -> Result<()> {
    let ty = sm.as_raw_sourcemap();
    serde_json::to_writer(&mut w, &ty)?;
    Ok(())
}

fn encode_vlq_diff(out: &mut String, a: u32, b: u32) {
    encode_vlq(out, i64::from(a) - i64::from(b))
}

fn encode_rmi(out: &mut Vec<u8>, indices: &[usize]) {
    fn encode_byte(b: u8) -> u8 {
        match b {
            0..=25 => b + b'A',
            26..=51 => b + b'a' - 26,
            52..=61 => b + b'0' - 52,
            62 => b'+',
            63 => b'/',
            _ => panic!("invalid byte"),
        }
    }

    let mut data = [0u8; 8];

    let bits = data.view_bits_mut::<Msb0>();
    for i in indices {
        bits.set(i - 1, true);
    }

    // trim zero at the end

    let mut last = 0;
    for (idx, bit) in bits.iter().enumerate() {
        if *bit {
            last = idx;
        }
    }
    let bits = &mut bits[..last + 1];

    for byte in bits.chunks_mut(6) {
        byte.reverse();

        let byte = byte.load::<u8>();

        let encoded = encode_byte(byte);

        out.push(encoded);
    }
}

fn serialize_range_mappings(sm: &SourceMap) -> Option<String> {
    if sm.range_tokens.is_empty() {
        return None;
    }

    let mut buf = Vec::new();
    let mut prev_line = 0;

    let mut idx_of_first_in_line = 0;
    let mut indices = vec![];

    for (idx, token) in sm.tokens().enumerate() {
        if token.is_range() {
            let num = idx - idx_of_first_in_line + 1;

            if num > 0 {
                indices.push(num);
            }
        }

        while token.get_dst_line() != prev_line {
            if !indices.is_empty() {
                encode_rmi(&mut buf, &indices);
                indices.clear();
            }

            buf.push(b';');
            prev_line += 1;
            idx_of_first_in_line = idx;
        }
    }
    if !indices.is_empty() {
        encode_rmi(&mut buf, &indices);
    }

    Some(unsafe {
        // Safety: We only push ASCII characters to the buffer
        String::from_utf8_unchecked(buf)
    })
}

fn serialize_mappings(sm: &SourceMap) -> String {
    let mut rv = String::new();
    // dst == minified == generated
    let mut prev_dst_line = 0;
    let mut prev_dst_col = 0;
    let mut prev_src_line = 0;
    let mut prev_src_col = 0;
    let mut prev_name_id = 0;
    let mut prev_src_id = 0;

    for (idx, token) in sm.tokens().enumerate() {
        let idx = idx as u32;

        if token.get_dst_line() != prev_dst_line {
            prev_dst_col = 0;
            while token.get_dst_line() != prev_dst_line {
                rv.push(';');
                prev_dst_line += 1;
            }
        } else if idx > 0 {
            if Some(&token) == sm.get_token(idx - 1).as_ref() {
                continue;
            }
            rv.push(',');
        }

        encode_vlq_diff(&mut rv, token.get_dst_col(), prev_dst_col);
        prev_dst_col = token.get_dst_col();

        if token.has_source() {
            encode_vlq_diff(&mut rv, token.get_src_id(), prev_src_id);
            prev_src_id = token.get_src_id();
            encode_vlq_diff(&mut rv, token.get_src_line(), prev_src_line);
            prev_src_line = token.get_src_line();
            encode_vlq_diff(&mut rv, token.get_src_col(), prev_src_col);
            prev_src_col = token.get_src_col();
            if token.has_name() {
                encode_vlq_diff(&mut rv, token.get_name_id(), prev_name_id);
                prev_name_id = token.get_name_id();
            }
        }
    }

    rv
}

impl Encodable for SourceMap {
    fn as_raw_sourcemap(&self) -> RawSourceMap {
        let mut have_contents = false;
        let contents = self
            .source_contents()
            .map(|contents| {
                if let Some(contents) = contents {
                    have_contents = true;
                    Some(contents.to_string())
                } else {
                    None
                }
            })
            .collect();
        RawSourceMap {
            version: Some(3),
            file: self.get_file().map(|x| Value::String(x.to_string())),
            sources: Some(self.sources.iter().map(|x| Some(x.to_string())).collect()),
            source_root: self.get_source_root().map(|x| x.to_string()),
            sources_content: if have_contents { Some(contents) } else { None },
            sections: None,
            names: Some(self.names().map(|x| Value::String(x.to_string())).collect()),
            range_mappings: dbg!(serialize_range_mappings(self)),
            mappings: Some(serialize_mappings(self)),
            x_facebook_offsets: None,
            x_metro_module_paths: None,
            x_facebook_sources: None,
            debug_id: self.get_debug_id(),
        }
    }
}

impl Encodable for SourceMapIndex {
    fn as_raw_sourcemap(&self) -> RawSourceMap {
        RawSourceMap {
            version: Some(3),
            file: self.get_file().map(|x| Value::String(x.to_string())),
            sources: None,
            source_root: None,
            sources_content: None,
            sections: Some(
                self.sections()
                    .map(|section| RawSection {
                        offset: RawSectionOffset {
                            line: section.get_offset_line(),
                            column: section.get_offset_col(),
                        },
                        url: section.get_url().map(str::to_owned),
                        map: section
                            .get_sourcemap()
                            .map(|sm| Box::new(sm.as_raw_sourcemap())),
                    })
                    .collect(),
            ),
            names: None,
            range_mappings: None,
            mappings: None,
            x_facebook_offsets: None,
            x_metro_module_paths: None,
            x_facebook_sources: None,
            debug_id: None,
        }
    }
}

impl Encodable for DecodedMap {
    fn as_raw_sourcemap(&self) -> RawSourceMap {
        match *self {
            DecodedMap::Regular(ref sm) => sm.as_raw_sourcemap(),
            DecodedMap::Index(ref smi) => smi.as_raw_sourcemap(),
            DecodedMap::Hermes(ref smh) => smh.as_raw_sourcemap(),
        }
    }
}
