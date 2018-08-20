#![allow(dead_code)]
#![allow(non_camel_case_types)]


extern crate jni_sys;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate combine;
extern crate cesu8;
//extern crate openssl;

mod jni;

extern crate actix;
extern crate actix_redis;
extern crate actix_web;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate jemalloc_ctl;
extern crate jemallocator;
extern crate libc;
extern crate http;
extern crate h2;

pub mod alloc;
pub mod buf;
pub mod listpack;
pub mod actor;
pub mod app;

/////////////////////////////////////////////////////////////////////////////
// Set Global Allocator to JEMalloc
/////////////////////////////////////////////////////////////////////////////

#[global_allocator]
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;