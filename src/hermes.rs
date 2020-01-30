use crate::decoder::{decode, decode_regular, decode_slice};
use crate::encoder::Encodable;
use crate::errors::{Error, Result};
use crate::jsontypes::FacebookScopeMapping;
use crate::jsontypes::RawSourceMap;
use crate::types::{DecodedMap, SourceMap};
use crate::vlq::parse_vlq_segment;
use std::cmp::Ordering;
use std::io::Read;
use std::ops::{Deref, DerefMut};

/// These are starting locations of scopes.
/// The `name_index` represents the index into the `HermesFunctionMap.names` vec,
/// which represents the function names/scopes.
pub struct HermesScopeOffset {
    line: u32,
    column: u32,
    name_index: u32,
}

pub struct HermesFunctionMap {
    names: Vec<String>,
    mappings: Vec<HermesScopeOffset>,
}

pub struct SourceMapHermes {
    pub(crate) sm: SourceMap,
    // There should be one `HermesFunctionMap` per each `sources` entry in the main SourceMap.
    function_maps: Vec<Option<HermesFunctionMap>>,
}

impl Deref for SourceMapHermes {
    type Target = SourceMap;

    fn deref(&self) -> &Self::Target {
        &self.sm
    }
}

impl DerefMut for SourceMapHermes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sm
    }
}

impl Encodable for SourceMapHermes {
    fn as_raw_sourcemap(&self) -> RawSourceMap {
        let rsm = self.sm.as_raw_sourcemap();
        // TODO: need to serialize the `HermesFunctionMap` mappings
        rsm
    }
}

impl SourceMapHermes {
    pub fn from_reader<R: Read>(rdr: R) -> Result<Self> {
        match decode(rdr)? {
            DecodedMap::Hermes(sm) => Ok(sm),
            _ => Err(Error::IndexedSourcemap),
        }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        match decode_slice(slice)? {
            DecodedMap::Hermes(sm) => Ok(sm),
            _ => Err(Error::IndexedSourcemap),
        }
    }

    pub fn get_original_function_name(&self, bytecode_offset: u32) -> Option<&str> {
        let token = self.sm.lookup_token(0, bytecode_offset)?;

        let function_map = &self.function_maps[token.get_src_id() as usize].as_ref()?;

        // Find the closest mapping, just like here:
        // https://github.com/facebook/metro/blob/63b523eb20e7bdf62018aeaf195bb5a3a1a67f36/packages/metro-symbolicate/src/SourceMetadataMapConsumer.js#L204-L231
        let mapping =
            function_map
                .mappings
                .binary_search_by(|o| match o.line.cmp(&token.get_src_line()) {
                    Ordering::Equal => o.column.cmp(&token.get_src_col()),
                    x => x,
                });
        let name_index = function_map.mappings[match mapping {
            Ok(a) => a,
            Err(a) => a.saturating_sub(1),
        }]
        .name_index;

        function_map
            .names
            .get(name_index as usize)
            .map(|n| n.as_str())
    }
}

pub fn decode_hermes(mut rsm: RawSourceMap) -> Result<SourceMapHermes> {
    let x_facebook_sources = rsm
        .x_facebook_sources
        .take()
        .expect("expected x_facebook_sources");

    // This is basically the logic from here:
    // https://github.com/facebook/metro/blob/63b523eb20e7bdf62018aeaf195bb5a3a1a67f36/packages/metro-symbolicate/src/SourceMetadataMapConsumer.js#L182-L202

    let function_maps = x_facebook_sources
        .into_iter()
        .map(|v| {
            let FacebookScopeMapping {
                names,
                mappings: raw_mappings,
            } = v.into_iter().next()?;

            let mut mappings = vec![];
            let mut line = 1;
            let mut name_index = 0;

            for line_mapping in raw_mappings.split(';') {
                if line_mapping.is_empty() {
                    continue;
                }

                let mut column = 0;

                for mapping in line_mapping.split(',') {
                    if mapping.is_empty() {
                        continue;
                    }

                    let mut nums = parse_vlq_segment(mapping).ok()?.into_iter();

                    column = (i64::from(column) + nums.next()?) as u32;
                    name_index = (i64::from(name_index) + nums.next().unwrap_or(0)) as u32;
                    line = (i64::from(line) + nums.next().unwrap_or(0)) as u32;
                    mappings.push(HermesScopeOffset {
                        column,
                        line,
                        name_index,
                    });
                }
            }
            Some(HermesFunctionMap { names, mappings })
        })
        .collect();

    let sm = decode_regular(rsm)?;
    Ok(SourceMapHermes { sm, function_maps })
}
