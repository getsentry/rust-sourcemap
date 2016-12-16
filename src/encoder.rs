use std::io::Write;

use serde_json;
use serde_json::Value;

use types::SourceMap;
use jsontypes::RawSourceMap;
use vlq::encode_vlq;
use errors::Result;

/// Represents things that can be encoded
pub trait Encodable {
    fn as_raw_sourcemap(&self) -> Result<RawSourceMap>;
}

pub fn encode<M: Encodable, W: Write>(sm: &M, mut w: W) -> Result<()> {
    let ty = try!(sm.as_raw_sourcemap());
    try!(serde_json::to_writer(&mut w, &ty));
    Ok(())
}

fn encode_vlq_diff(out: &mut String, a: u32, b: u32) {
    encode_vlq(out, (a as i64) - (b as i64))
}

fn serialize_mappings(sm: &SourceMap) -> Result<String> {
    let mut rv = String::new();
    // dst == minified == generated
    let mut prev_dst_line = 0;
    let mut prev_dst_col = 0;
    let mut prev_src_line = 0;
    let mut prev_src_col = 0;
    let mut prev_name_id = 0;
    let mut prev_src_id = 0;

    for (idx, token) in sm.tokens().enumerate() {
        let raw = token.get_raw_token();
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
            encode_vlq_diff(&mut rv, raw.src_id, prev_src_id);
            prev_src_id = raw.src_id;
            encode_vlq_diff(&mut rv, token.get_src_line(), prev_src_line);
            prev_src_line = token.get_src_line();
            encode_vlq_diff(&mut rv, token.get_src_col(), prev_src_col);
            prev_src_col = token.get_src_col();
            if token.has_name() {
                encode_vlq_diff(&mut rv, raw.name_id, prev_name_id);
                prev_name_id = raw.name_id;
            }
        }
    }

    Ok(rv)
}

impl Encodable for SourceMap {
    fn as_raw_sourcemap(&self) -> Result<RawSourceMap> {
        let mut have_contents = false;
        let contents = self.source_contents().map(|contents| {
            if let Some(contents) = contents {
                have_contents = true;
                Some(contents.to_string())
            } else {
                None
            }
        }).collect();
        Ok(RawSourceMap {
            version: Some(self.get_version()),
            file: self.get_file().map(|x| Value::String(x.to_string())),
            sources: Some(self.sources().map(|x| x.to_string()).collect()),
            // XXX: consider setting this to common root
            source_root: None,
            sources_content: if have_contents {
                Some(contents)
            } else {
                None
            },
            sections: None,
            names: Some(self.names().map(|x| Value::String(x.to_string())).collect()),
            mappings: Some(try!(serialize_mappings(self))),
        })
    }
}
