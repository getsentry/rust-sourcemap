#[cfg(feature="serde_macros")]
include!("jsontypes.rs.in");

#[cfg(not(feature="serde_macros"))]
include!(concat!(env!("OUT_DIR"), "/jsontypes.rs"));
