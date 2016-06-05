#![cfg_attr(feature="serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature="serde_macros", plugin(serde_macros))]

#[cfg(feature="serde_macros")]
include!("jsontypes.rs.in");

#[cfg(not(feature="serde_macros"))]
include!(concat!(env!("OUT_DIR"), "/jsontypes.rs"));
