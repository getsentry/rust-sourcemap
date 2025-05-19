use std::sync::Arc;

use debugid::DebugId;
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils::intern;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RawSectionOffset {
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RawSection {
    pub offset: RawSectionOffset,
    pub url: Option<String>,
    pub map: Option<Box<RawSourceMap>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FacebookScopeMapping {
    pub names: Vec<String>,
    pub mappings: String,
}

// Each element here is matching the `sources` of the outer SourceMap.
// It has a list of metadata, the first one of which is a *function map*,
// containing scope information as a nested source map.
// See the decoder in `hermes.rs` for details.
pub type FacebookSources = Option<Vec<Option<Vec<FacebookScopeMapping>>>>;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct InterningArc(pub(crate) Arc<str>);

impl From<InterningArc> for Arc<str> {
    fn from(value: InterningArc) -> Self {
        value.0
    }
}

impl From<Arc<str>> for InterningArc {
    fn from(value: Arc<str>) -> Self {
        InterningArc(value)
    }
}

impl<'de> Deserialize<'de> for InterningArc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(InterningArc(intern(s)))
    }
}

impl Serialize for InterningArc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RawSourceMap {
    pub version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Value>,
    pub sources: Option<Vec<Option<InterningArc>>>,
    #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
    pub source_root: Option<String>,
    #[serde(rename = "sourcesContent", skip_serializing_if = "Option::is_none")]
    pub sources_content: Option<Vec<Option<InterningArc>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<RawSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<Value>>,
    #[serde(rename = "rangeMappings", skip_serializing_if = "Option::is_none")]
    pub range_mappings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappings: Option<String>,
    #[serde(rename = "ignoreList", skip_serializing_if = "Option::is_none")]
    pub ignore_list: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_facebook_offsets: Option<Vec<Option<u32>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_metro_module_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_facebook_sources: FacebookSources,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_id: Option<DebugId>,
    // This field only exists to be able to deserialize from "debugId" keys
    // if "debug_id" is unset.
    #[serde(skip_serializing_if = "Option::is_none", rename = "debugId")]
    pub(crate) _debug_id_new: Option<DebugId>,
}

#[derive(Deserialize)]
pub struct MinimalRawSourceMap {
    pub version: Option<u32>,
    pub file: Option<IgnoredAny>,
    pub sources: Option<IgnoredAny>,
    #[serde(rename = "sourceRoot")]
    pub source_root: Option<IgnoredAny>,
    #[serde(rename = "sourcesContent")]
    pub sources_content: Option<IgnoredAny>,
    pub sections: Option<IgnoredAny>,
    pub names: Option<IgnoredAny>,
    pub mappings: Option<IgnoredAny>,
}
