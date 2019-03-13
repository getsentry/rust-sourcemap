use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct RawSectionOffset {
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize, Deserialize)]
pub struct RawSection {
    pub offset: RawSectionOffset,
    pub url: Option<String>,
    pub map: Option<Box<RawSourceMap>>,
}

#[derive(Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mappings: Option<String>,
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
