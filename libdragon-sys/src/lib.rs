#![no_std]
#![feature(alloc_error_handler)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]
// disable some clippy suggestions for generated code
#![allow(clippy::useless_transmute)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::approx_constant)]
#![allow(clippy::missing_safety_doc)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
