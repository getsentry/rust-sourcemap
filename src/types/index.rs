use std::{
    borrow::Cow,
    io::{Read, Write},
};

use debugid::DebugId;

use crate::{
    decoder, encoder, utils::greatest_lower_bound, DecodedMap, Error, Result, RewriteOptions,
    SourceMap, SourceMapBuilder, SourceView, Token,
};

/// Represents a sourcemap index in memory
#[derive(Debug, Clone, PartialEq)]
pub struct SourceMapIndex {
    file: Option<String>,
    sections: Vec<SourceMapSection>,
    x_facebook_offsets: Option<Vec<Option<u32>>>,
    x_metro_module_paths: Option<Vec<String>>,
    debug_id: Option<DebugId>,
}

impl SourceMapIndex {
    /// Creates a sourcemap index from a reader over a JSON stream in UTF-8
    /// format.  Optionally a "garbage header" as defined by the
    /// sourcemap draft specification is supported.  In case a regular
    /// sourcemap is encountered an error is returned.
    pub fn from_reader<R: Read>(rdr: R) -> Result<SourceMapIndex> {
        match decoder::decode(rdr)? {
            DecodedMap::Index(smi) => Ok(smi),
            _ => Err(Error::IncompatibleSourceMap),
        }
    }

    /// Writes a sourcemap index into a writer.
    pub fn to_writer<W: Write>(&self, w: W) -> Result<()> {
        encoder::encode(self, w)
    }

    /// Creates a sourcemap index from a reader over a JSON byte slice in UTF-8
    /// format.  Optionally a "garbage header" as defined by the
    /// sourcemap draft specification is supported.  In case a regular
    /// sourcemap is encountered an error is returned.
    pub fn from_slice(slice: &[u8]) -> Result<SourceMapIndex> {
        match decoder::decode_slice(slice)? {
            DecodedMap::Index(smi) => Ok(smi),
            _ => Err(Error::IncompatibleSourceMap),
        }
    }

    /// Constructs a new sourcemap index from raw components.
    ///
    /// - `file`: an optional filename of the index
    /// - `sections`: a vector of source map index sections
    pub fn new(file: Option<String>, sections: Vec<SourceMapSection>) -> SourceMapIndex {
        SourceMapIndex {
            file,
            sections,
            x_facebook_offsets: None,
            x_metro_module_paths: None,
            debug_id: None,
        }
    }

    /// Constructs a new sourcemap index from raw components including the
    /// facebook RAM bundle extensions.
    ///
    /// - `file`: an optional filename of the index
    /// - `sections`: a vector of source map index sections
    /// - `x_facebook_offsets`: a vector of facebook offsets
    /// - `x_metro_module_paths`: a vector of metro module paths
    pub fn new_ram_bundle_compatible(
        file: Option<String>,
        sections: Vec<SourceMapSection>,
        x_facebook_offsets: Option<Vec<Option<u32>>>,
        x_metro_module_paths: Option<Vec<String>>,
    ) -> SourceMapIndex {
        SourceMapIndex {
            file,
            sections,
            x_facebook_offsets,
            x_metro_module_paths,
            debug_id: None,
        }
    }

    /// Returns the debug ID.
    pub(crate) fn debug_id(&self) -> Option<DebugId> {
        self.debug_id
    }

    pub(crate) fn set_debug_id(&mut self, debug_id: Option<DebugId>) {
        self.debug_id = debug_id;
    }

    /// Adds the given debug id to the sourcemap index.
    pub(crate) fn with_debug_id(mut self, debug_id: Option<DebugId>) -> Self {
        self.set_debug_id(debug_id);
        self
    }

    /// Returns the embedded filename in case there is one.
    pub fn get_file(&self) -> Option<&str> {
        self.file.as_ref().map(|x| &x[..])
    }

    /// Sets a new value for the file.
    pub fn set_file(&mut self, value: Option<&str>) {
        self.file = value.map(str::to_owned);
    }

    /// Returns the number of sections in this index
    pub fn get_section_count(&self) -> u32 {
        self.sections.len() as u32
    }

    /// Looks up a single section and returns it
    pub fn get_section(&self, idx: u32) -> Option<&SourceMapSection> {
        self.sections.get(idx as usize)
    }

    /// Looks up a single section and returns it as a mutable ref
    pub fn get_section_mut(&mut self, idx: u32) -> Option<&mut SourceMapSection> {
        self.sections.get_mut(idx as usize)
    }

    /// Iterates over all sections
    pub fn sections(&self) -> SourceMapSectionIter<'_> {
        SourceMapSectionIter {
            i: self,
            next_idx: 0,
        }
    }

    /// Given a location, name and minified source file resolve a minified
    /// name to an original function name.
    ///
    /// This invokes some guesswork and requires access to the original minified
    /// source.  This will not yield proper results for anonymous functions or
    /// functions that do not have clear function names.  (For instance it's
    /// recommended that dotted function names are not passed to this
    /// function).
    pub fn get_original_function_name(
        &self,
        line: u32,
        col: u32,
        minified_name: &str,
        sv: &SourceView,
    ) -> Option<&str> {
        self.lookup_token(line, col)
            .and_then(|token| sv.get_original_function_name(token, minified_name))
    }

    /// Looks up the closest token to a given line and column.
    ///
    /// This requires that the referenced sourcemaps are actually loaded.
    /// If a sourcemap is encountered that is not embedded but just
    /// externally referenced it is silently skipped.
    pub fn lookup_token(&self, line: u32, col: u32) -> Option<Token<'_>> {
        let (_section_idx, section) =
            greatest_lower_bound(&self.sections, &(line, col), SourceMapSection::get_offset)?;
        let map = section.get_sourcemap()?;
        let (off_line, off_col) = section.get_offset();
        map.lookup_token(
            line - off_line,
            if line == off_line { col - off_col } else { col },
        )
    }

    /// Flattens an indexed sourcemap into a regular one.  This requires
    /// that all referenced sourcemaps are attached.
    pub fn flatten(&self) -> Result<SourceMap> {
        let mut builder = SourceMapBuilder::new(self.get_file());

        for section in self.sections() {
            let (off_line, off_col) = section.get_offset();
            let map = match section.get_sourcemap() {
                Some(map) => match map {
                    DecodedMap::Regular(sm) => Cow::Borrowed(sm),
                    DecodedMap::Index(idx) => Cow::Owned(idx.flatten()?),
                    DecodedMap::Hermes(smh) => Cow::Borrowed(&smh.sm),
                },
                None => {
                    return Err(Error::CannotFlatten(format!(
                        "Section has an unresolved \
                         sourcemap: {}",
                        section.get_url().unwrap_or("<unknown url>")
                    )));
                }
            };

            let mut src_id_map = Vec::<u32>::with_capacity(map.sources().count());

            for (original_id, (source, contents)) in
                map.sources().zip(map.source_contents()).enumerate()
            {
                debug_assert_eq!(original_id, src_id_map.len());
                let src_id = builder.add_source(source);

                src_id_map.push(src_id);

                if let Some(contents) = contents {
                    builder.set_source_contents(src_id, Some(contents));
                }
            }

            let mut name_id_map = Vec::<u32>::with_capacity(map.names().count());

            for (original_id, name) in map.names().enumerate() {
                debug_assert_eq!(original_id, name_id_map.len());
                let name_id = builder.add_name(name);
                name_id_map.push(name_id);
            }

            for token in map.tokens() {
                let dst_col = if token.get_dst_line() == 0 {
                    token.get_dst_col() + off_col
                } else {
                    token.get_dst_col()
                };

                // Use u32 -> u32 map instead of using the hash map in SourceMapBuilder for better performance
                let original_src_id = token.get_src_id();
                let src_id = if original_src_id == !0 {
                    None
                } else {
                    src_id_map.get(original_src_id as usize).copied()
                };

                let original_name_id = token.get_name_id();
                let name_id = if original_name_id == !0 {
                    None
                } else {
                    name_id_map.get(original_name_id as usize).copied()
                };

                let raw = builder.add_raw(
                    token.get_dst_line() + off_line,
                    dst_col,
                    token.get_src_line(),
                    token.get_src_col(),
                    src_id,
                    name_id,
                    token.is_range(),
                );

                if map.ignore_list.contains(&token.get_src_id()) {
                    builder.add_to_ignore_list(raw.src_id);
                }
            }
        }

        Ok(builder.into_sourcemap())
    }

    /// Flattens an indexed sourcemap into a regular one and automatically
    /// rewrites it.  This is more useful than plain flattening as this will
    /// cause the sourcemap to be properly deduplicated.
    pub fn flatten_and_rewrite(self, options: &RewriteOptions<'_>) -> Result<SourceMap> {
        self.flatten()?.rewrite(options)
    }

    /// Returns `true` if this sourcemap is for a RAM bundle.
    pub fn is_for_ram_bundle(&self) -> bool {
        self.x_facebook_offsets.is_some() && self.x_metro_module_paths.is_some()
    }

    /// Returns embeded x-facebook-offset values.
    pub fn x_facebook_offsets(&self) -> Option<&[Option<u32>]> {
        self.x_facebook_offsets.as_ref().map(|x| &x[..])
    }

    /// Returns embedded metro module paths.
    pub fn x_metro_module_paths(&self) -> Option<&[String]> {
        self.x_metro_module_paths.as_ref().map(|x| &x[..])
    }

    /// Adjusts all of the sections' offset rows by the given amount.
    /// Returns a boolean indicating whether the adjustment was successful
    /// (false indicating that not all of the sections could be adjusted
    /// because we overflowed the u32, true if adjustment was successful).
    /// If false is returned, then the sourcemap index is unchanged.
    pub fn adjust_sections_offset_rows(&mut self, amount: u32) -> bool {
        let adjusted_rows: Vec<_> = self
            .sections
            .iter()
            // Filter map will filter out adjustments that overflow
            .filter_map(|section| section.offset.0.checked_add(amount))
            .collect();

        if adjusted_rows.len() != self.sections.len() {
            // We overflowed at least one section
            return false;
        }

        for (section, adjustment) in self.sections.iter_mut().zip(adjusted_rows) {
            section.offset.0 = adjustment;
        }

        true
    }
}

/// Represents a section in a sourcemap index
#[derive(Debug, Clone, PartialEq)]
pub struct SourceMapSection {
    offset: (u32, u32),
    url: Option<String>,
    map: Option<Box<DecodedMap>>,
}

impl SourceMapSection {
    /// Create a new sourcemap index section
    ///
    /// - `offset`: offset as line and column
    /// - `url`: optional URL of where the sourcemap is located
    /// - `map`: an optional already resolved internal sourcemap
    pub fn new(
        offset: (u32, u32),
        url: Option<String>,
        map: Option<DecodedMap>,
    ) -> SourceMapSection {
        SourceMapSection {
            offset,
            url,
            map: map.map(Box::new),
        }
    }

    /// Returns the offset line
    pub fn get_offset_line(&self) -> u32 {
        self.offset.0
    }

    /// Returns the offset column
    pub fn get_offset_col(&self) -> u32 {
        self.offset.1
    }

    /// Returns the offset as tuple
    pub fn get_offset(&self) -> (u32, u32) {
        self.offset
    }

    /// Returns the URL of the referenced map if available
    pub fn get_url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Updates the URL for this section.
    pub fn set_url(&mut self, value: Option<&str>) {
        self.url = value.map(str::to_owned);
    }

    /// Returns a reference to the embedded sourcemap if available
    pub fn get_sourcemap(&self) -> Option<&DecodedMap> {
        self.map.as_ref().map(Box::as_ref)
    }

    /// Returns a reference to the embedded sourcemap if available
    pub fn get_sourcemap_mut(&mut self) -> Option<&mut DecodedMap> {
        self.map.as_mut().map(Box::as_mut)
    }

    /// Replaces the embedded sourcemap
    pub fn set_sourcemap(&mut self, sm: Option<DecodedMap>) {
        self.map = sm.map(Box::new);
    }
}

/// Iterates over all sections in a sourcemap index
pub struct SourceMapSectionIter<'a> {
    i: &'a SourceMapIndex,
    next_idx: u32,
}

impl<'a> Iterator for SourceMapSectionIter<'a> {
    type Item = &'a SourceMapSection;

    fn next(&mut self) -> Option<&'a SourceMapSection> {
        self.i.get_section(self.next_idx).inspect(|_| {
            self.next_idx += 1;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sourcemap_index_default_debug_id() {
        let sm = SourceMapIndex::new(None, vec![]);
        assert!(sm.debug_id().is_none());
    }

    #[test]
    fn test_sourcemap_index_debug_id() {
        const DEBUG_ID: &str = "0123456789abcdef0123456789abcdef";

        let sm = SourceMapIndex::new(None, vec![])
            .with_debug_id(Some(DEBUG_ID.parse().expect("valid debug id")));

        assert_eq!(
            sm.debug_id(),
            Some(DEBUG_ID.parse().expect("valid debug id"))
        );
    }

    #[test]
    fn test_adjust_sections_offset_rows_basic() {
        // Create a sourcemap index with sections starting at (0, 0) and (10, 0)
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((10, 0), None, None),
            ],
        );

        // Adjust by 1
        assert!(smi.adjust_sections_offset_rows(1));

        // Check that the entire SourceMapIndex was adjusted correctly
        assert_eq!(
            smi,
            SourceMapIndex::new(
                Some("test.js".to_string()),
                vec![
                    SourceMapSection::new((1, 0), None, None),
                    SourceMapSection::new((11, 0), None, None),
                ],
            )
        );
    }

    #[test]
    fn test_adjust_sections_offset_rows_zero() {
        // Create a sourcemap index with sections starting at (0, 0) and (10, 0)
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((10, 0), None, None),
            ],
        );

        // Adjust by zero
        assert!(smi.adjust_sections_offset_rows(0));

        // Check that the entire SourceMapIndex remained unchanged
        assert_eq!(
            smi,
            SourceMapIndex::new(
                Some("test.js".to_string()),
                vec![
                    SourceMapSection::new((0, 0), None, None),
                    SourceMapSection::new((10, 0), None, None),
                ],
            )
        );
    }

    #[test]
    fn test_adjust_sections_offset_rows_multiple_sections() {
        // Create a sourcemap index with multiple sections
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((10, 0), None, None),
                SourceMapSection::new((20, 10), None, None),
                SourceMapSection::new((30, 40), None, None),
            ],
        );

        // Adjust by 1
        assert!(smi.adjust_sections_offset_rows(1));

        // Check that the entire SourceMapIndex was adjusted correctly
        assert_eq!(
            smi,
            SourceMapIndex::new(
                Some("test.js".to_string()),
                vec![
                    SourceMapSection::new((1, 0), None, None),
                    SourceMapSection::new((11, 0), None, None),
                    SourceMapSection::new((21, 10), None, None),
                    SourceMapSection::new((31, 40), None, None),
                ],
            )
        );
    }

    #[test]
    fn test_adjust_sections_offset_rows_overflow() {
        // Create a sourcemap index with a section at u32::MAX
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((u32::MAX, 0), None, None),
            ],
        );

        // Store the original state
        let original_smi = smi.clone();

        // An adjustment of 1 would overflow
        assert!(!smi.adjust_sections_offset_rows(1));

        // Verify the sourcemap index remains unchanged
        assert_eq!(smi, original_smi);
    }

    #[test]
    fn test_adjust_sections_offset_rows_partial_overflow() {
        // Create a sourcemap index with multiple sections, one at u32::MAX
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((10, 0), None, None),
                SourceMapSection::new((20, 0), None, None),
                SourceMapSection::new((u32::MAX, 0), None, None),
            ],
        );

        // Store the original state
        let original_smi = smi.clone();

        // Try to adjust by an amount that would cause overflow for one section
        assert!(!smi.adjust_sections_offset_rows(1));

        // Verify the sourcemap index remains unchanged
        assert_eq!(smi, original_smi);
    }

    #[test]
    fn test_adjust_sections_offset_rows_large_amount() {
        // Create a sourcemap index with sections
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((10, 0), None, None),
            ],
        );

        assert!(smi.adjust_sections_offset_rows(1_000_000));

        // Check that the entire SourceMapIndex was adjusted correctly
        assert_eq!(
            smi,
            SourceMapIndex::new(
                Some("test.js".to_string()),
                vec![
                    SourceMapSection::new((1_000_000, 0), None, None),
                    SourceMapSection::new((1_000_010, 0), None, None),
                ],
            )
        );
    }

    #[test]
    fn adjust_sections_offset_rows_large_amount_overflow() {
        // Create a sourcemap index with a section at a positive amount
        let mut smi = SourceMapIndex::new(
            Some("test.js".to_string()),
            vec![
                SourceMapSection::new((0, 0), None, None),
                SourceMapSection::new((10, 0), None, None),
            ],
        );

        // Store the original state
        let original_smi = smi.clone();

        // An adjustment of u32::MAX would overflow
        assert!(!smi.adjust_sections_offset_rows(u32::MAX));

        // Verify the sourcemap index remains unchanged
        assert_eq!(smi, original_smi);
    }

    #[test]
    fn adjust_sections_offset_rows_no_sections() {
        // Create a sourcemap index with no sections
        let mut smi = SourceMapIndex::new(Some("test.js".to_string()), vec![]);

        // An adjustment by 1 should return true and no-op
        assert!(smi.adjust_sections_offset_rows(1));

        // The sourcemap index should remain unchanged
        assert_eq!(
            smi,
            SourceMapIndex::new(Some("test.js".to_string()), vec![])
        );
    }
}
