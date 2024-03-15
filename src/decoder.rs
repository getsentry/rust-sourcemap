use std::io;
use std::io::{BufReader, Read};

use serde_json::Value;

use crate::errors::{Error, Result};
use crate::hermes::decode_hermes;
use crate::jsontypes::RawSourceMap;
use crate::types::{DecodedMap, RawToken, SourceMap, SourceMapIndex, SourceMapSection};
use crate::vlq::parse_vlq_segment_into;

const DATA_PREAMBLE: &str = "data:application/json;base64,";

#[derive(PartialEq, Eq)]
enum HeaderState {
    Undecided,
    Junk,
    AwaitingNewline,
    PastHeader,
}

pub struct StripHeaderReader<R: Read> {
    r: R,
    header_state: HeaderState,
}

impl<R: Read> StripHeaderReader<R> {
    pub fn new(reader: R) -> StripHeaderReader<R> {
        StripHeaderReader {
            r: reader,
            header_state: HeaderState::Undecided,
        }
    }
}

fn is_junk_json(byte: u8) -> bool {
    byte == b')' || byte == b']' || byte == b'}' || byte == b'\''
}

impl<R: Read> Read for StripHeaderReader<R> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.header_state == HeaderState::PastHeader {
            return self.r.read(buf);
        }
        self.strip_head_read(buf)
    }
}

impl<R: Read> StripHeaderReader<R> {
    fn strip_head_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut backing = vec![0; buf.len()];
        let local_buf: &mut [u8] = &mut backing;

        loop {
            let read = self.r.read(local_buf)?;
            if read == 0 {
                return Ok(0);
            }
            for (offset, &byte) in local_buf[0..read].iter().enumerate() {
                self.header_state = match self.header_state {
                    HeaderState::Undecided => {
                        if is_junk_json(byte) {
                            HeaderState::Junk
                        } else {
                            buf[..read].copy_from_slice(&local_buf[..read]);
                            self.header_state = HeaderState::PastHeader;
                            return Ok(read);
                        }
                    }
                    HeaderState::Junk => {
                        if byte == b'\r' {
                            HeaderState::AwaitingNewline
                        } else if byte == b'\n' {
                            HeaderState::PastHeader
                        } else {
                            HeaderState::Junk
                        }
                    }
                    HeaderState::AwaitingNewline => {
                        if byte == b'\n' {
                            HeaderState::PastHeader
                        } else {
                            fail!(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "expected newline"
                            ));
                        }
                    }
                    HeaderState::PastHeader => {
                        let rem = read - offset;
                        buf[..rem].copy_from_slice(&local_buf[offset..read]);
                        return Ok(rem);
                    }
                };
            }
        }
    }
}

pub fn strip_junk_header(slice: &[u8]) -> io::Result<&[u8]> {
    if slice.is_empty() || !is_junk_json(slice[0]) {
        return Ok(slice);
    }
    let mut need_newline = false;
    for (idx, &byte) in slice.iter().enumerate() {
        if need_newline && byte != b'\n' {
            fail!(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected newline"
            ));
        } else if is_junk_json(byte) {
            continue;
        } else if byte == b'\r' {
            need_newline = true;
        } else if byte == b'\n' {
            return Ok(&slice[idx..]);
        }
    }
    Ok(&slice[slice.len()..])
}

pub fn decode_regular(rsm: RawSourceMap) -> Result<SourceMap> {
    let mut dst_col;
    let mut src_id = 0;
    let mut src_line = 0;
    let mut src_col = 0;
    let mut name_id = 0;

    let names = rsm.names.unwrap_or_default();
    let sources = rsm.sources.unwrap_or_default();
    let mappings = rsm.mappings.unwrap_or_default();
    let allocation_size = mappings.matches(&[',', ';'][..]).count() + 10;
    let mut tokens = Vec::with_capacity(allocation_size);

    let mut nums = Vec::with_capacity(6);

    for (dst_line, line) in mappings.split(';').enumerate() {
        if line.is_empty() {
            continue;
        }

        dst_col = 0;

        for segment in line.split(',') {
            if segment.is_empty() {
                continue;
            }

            nums.clear();
            parse_vlq_segment_into(segment, &mut nums)?;
            dst_col = (i64::from(dst_col) + nums[0]) as u32;

            let mut src = !0;
            let mut name = !0;

            if nums.len() > 1 {
                if nums.len() != 4 && nums.len() != 5 {
                    fail!(Error::BadSegmentSize(nums.len() as u32));
                }
                src_id = (i64::from(src_id) + nums[1]) as u32;
                if src_id >= sources.len() as u32 {
                    fail!(Error::BadSourceReference(src_id));
                }

                src = src_id;
                src_line = (i64::from(src_line) + nums[2]) as u32;
                src_col = (i64::from(src_col) + nums[3]) as u32;

                if nums.len() > 4 {
                    name_id = (i64::from(name_id) + nums[4]) as u32;
                    if name_id >= names.len() as u32 {
                        fail!(Error::BadNameReference(name_id));
                    }
                    name = name_id;
                }
            }

            tokens.push(RawToken {
                dst_line: dst_line as u32,
                dst_col,
                src_line,
                src_col,
                src_id: src,
                name_id: name,
            });
        }
    }

    let sources = sources
        .into_iter()
        .map(Option::unwrap_or_default)
        .map(Into::into)
        .collect();

    // apparently we can encounter some non string types in real world
    // sourcemaps :(
    let names = names
        .into_iter()
        .map(|val| match val {
            Value::String(s) => s.into(),
            Value::Number(num) => num.to_string().into(),
            _ => "".into(),
        })
        .collect::<Vec<_>>();

    // file sometimes is not a string for unexplicable reasons
    let file = rsm.file.map(|val| match val {
        Value::String(s) => s.into(),
        _ => "<invalid>".into(),
    });

    let source_content = rsm
        .sources_content
        .map(|x| x.into_iter().map(|v| v.map(Into::into)).collect::<Vec<_>>());

    let mut sm = SourceMap::new(file, tokens, names, sources, source_content);
    sm.set_source_root(rsm.source_root);
    sm.set_debug_id(rsm.debug_id);

    Ok(sm)
}

fn decode_index(rsm: RawSourceMap) -> Result<SourceMapIndex> {
    let mut sections = vec![];

    for mut raw_section in rsm.sections.unwrap_or_default() {
        sections.push(SourceMapSection::new(
            (raw_section.offset.line, raw_section.offset.column),
            raw_section.url,
            match raw_section.map.take() {
                Some(map) => Some(decode_common(*map)?),
                None => None,
            },
        ));
    }

    sections.sort_by_key(SourceMapSection::get_offset);

    // file sometimes is not a string for unexplicable reasons
    let file = rsm.file.map(|val| match val {
        Value::String(s) => s,
        _ => "<invalid>".into(),
    });

    Ok(SourceMapIndex::new_ram_bundle_compatible(
        file,
        sections,
        rsm.x_facebook_offsets,
        rsm.x_metro_module_paths,
    ))
}

fn decode_common(rsm: RawSourceMap) -> Result<DecodedMap> {
    Ok(if rsm.sections.is_some() {
        DecodedMap::Index(decode_index(rsm)?)
    } else if rsm.x_facebook_sources.is_some() {
        DecodedMap::Hermes(decode_hermes(rsm)?)
    } else {
        DecodedMap::Regular(decode_regular(rsm)?)
    })
}

/// Decodes a sourcemap or sourcemap index from a reader
///
/// This supports both sourcemaps and sourcemap indexes unless the
/// specialized methods on the individual types.
pub fn decode<R: Read>(rdr: R) -> Result<DecodedMap> {
    let mut rdr = StripHeaderReader::new(rdr);
    let mut rdr = BufReader::new(&mut rdr);
    let rsm: RawSourceMap = serde_json::from_reader(&mut rdr)?;
    decode_common(rsm)
}

/// Decodes a sourcemap or sourcemap index from a byte slice
///
/// This supports both sourcemaps and sourcemap indexes unless the
/// specialized methods on the individual types.
pub fn decode_slice(slice: &[u8]) -> Result<DecodedMap> {
    let content = strip_junk_header(slice)?;
    let rsm: RawSourceMap = serde_json::from_slice(content)?;
    decode_common(rsm)
}

/// Loads a sourcemap from a data URL
pub fn decode_data_url(url: &str) -> Result<DecodedMap> {
    if !url.starts_with(DATA_PREAMBLE) {
        fail!(Error::InvalidDataUrl);
    }
    let data_b64 = &url[DATA_PREAMBLE.len()..];
    let data = data_encoding::BASE64
        .decode(data_b64.as_bytes())
        .map_err(|_| Error::InvalidDataUrl)?;
    decode_slice(&data[..])
}

#[test]
fn test_strip_header() {
    use std::io::BufRead;
    let input: &[_] = b")]}garbage\r\n[1, 2, 3]";
    let mut reader = io::BufReader::new(StripHeaderReader::new(input));
    let mut text = String::new();
    reader.read_line(&mut text).ok();
    assert_eq!(text, "[1, 2, 3]");
}

#[test]
fn test_bad_newline() {
    use std::io::BufRead;
    let input: &[_] = b")]}'\r[1, 2, 3]";
    let mut reader = io::BufReader::new(StripHeaderReader::new(input));
    let mut text = String::new();
    match reader.read_line(&mut text) {
        Err(err) => {
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        }
        Ok(_) => {
            panic!("Expected failure");
        }
    }
}
