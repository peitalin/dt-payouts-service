#![allow(unused_imports)]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![recursion_limit = "128"]

extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
#[macro_use]
extern crate log;
extern crate num;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate uuid;

// Internal modules
pub mod db;
pub mod models;
pub mod utils;


