#[cfg(feature="serde_derive")]
include!("jsontypes.rs.in");

#[cfg(not(feature="serde_derive"))]
include!(concat!(env!("OUT_DIR"), "/jsontypes.rs"));
