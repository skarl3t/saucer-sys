#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused)]

#[cfg(feature = "gen-bindings")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "gen-bindings"))]
include!("bindings.rs");
