use bytes_str::BytesStr;
use debugid::DebugId;
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt::Debug;

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
    pub names: Vec<BytesStr>,
    pub mappings: String,
}

// Each element here is matching the `sources` of the outer SourceMap.
// It has a list of metadata, the first one of which is a *function map*,
// containing scope information as a nested source map.
// See the decoder in `hermes.rs` for details.
pub type FacebookSources = Option<Vec<Option<Vec<FacebookScopeMapping>>>>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RawSourceMap {
    pub version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Value>,
    pub sources: Option<Vec<Option<String>>>,
    #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
    pub source_root: Option<String>,
    #[serde(rename = "sourcesContent", skip_serializing_if = "Option::is_none")]
    pub sources_content: Option<Vec<Option<String>>>,
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
    #[serde(flatten)]
    pub debug_id: DebugIdField,
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

/// This struct represents a `RawSourceMap`'s debug ID fields.
///
/// The reason this exists as a seperate struct is so that we can have custom deserialization
/// logic, which can read both the  legacy snake_case debug_id and the new camelCase debugId
/// fields. In case both are provided, the camelCase field takes precedence.
///
/// The field is always serialized as `debugId`.
#[derive(Serialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct DebugIdField {
    #[serde(rename = "debugId", skip_serializing_if = "Option::is_none")]
    value: Option<DebugId>,
}

impl<'de> Deserialize<'de> for DebugIdField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We cannot use serde(alias), as that would cause an error when both fields are present.

        #[derive(Deserialize)]
        struct Helper {
            #[serde(rename = "debugId")]
            camel: Option<DebugId>,
            #[serde(rename = "debug_id")]
            legacy: Option<DebugId>,
        }

        let Helper { camel, legacy } = Helper::deserialize(deserializer)?;
        Ok(camel.or(legacy).into())
    }
}

impl From<Option<DebugId>> for DebugIdField {
    fn from(value: Option<DebugId>) -> Self {
        Self { value }
    }
}

impl From<DebugIdField> for Option<DebugId> {
    fn from(value: DebugIdField) -> Self {
        value.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_debug_id(input: &str) -> DebugId {
        input.parse().expect("valid debug id")
    }

    fn empty_sourcemap() -> RawSourceMap {
        serde_json::from_value::<RawSourceMap>(serde_json::json!({}))
            .expect("can deserialize empty JSON to RawSourceMap")
    }

    #[test]
    fn raw_sourcemap_serializes_camel_case_debug_id() {
        let camel = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
        let raw = RawSourceMap {
            debug_id: Some(parse_debug_id(camel)).into(),
            ..empty_sourcemap()
        };

        let value = serde_json::to_value(raw).expect("should serialize without error");
        let obj = value.as_object().expect("should be an object");
        assert!(obj.get("debug_id").is_none());
        assert_eq!(obj.get("debugId"), Some(&json!(parse_debug_id(camel))));
    }

    #[test]
    fn raw_sourcemap_prefers_camel_case_on_deserialize() {
        let legacy = "ffffffffffffffffffffffffffffffff";
        let camel = "00000000000000000000000000000000";
        let json = serde_json::json!({
            "debug_id": legacy,
            "debugId": camel
        });
        let raw: RawSourceMap =
            serde_json::from_value(json).expect("can deserialize as RawSourceMap");
        let value: Option<DebugId> = raw.debug_id.into();
        assert_eq!(value, Some(parse_debug_id(camel)));
    }
}
