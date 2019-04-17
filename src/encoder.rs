use std::io::Write;

use serde_json;
use serde_json::Value;

use crate::errors::Result;
use crate::jsontypes::{RawSection, RawSectionOffset, RawSourceMap};
use crate::types::{SourceMap, SourceMapIndex};
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
            sources: Some(self.sources().map(|x| Some(x.to_string())).collect()),
            // XXX: consider setting this to common root
            source_root: None,
            sources_content: if have_contents { Some(contents) } else { None },
            sections: None,
            names: Some(self.names().map(|x| Value::String(x.to_string())).collect()),
            mappings: Some(serialize_mappings(self)),
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
            mappings: None,
        }
    }
}
