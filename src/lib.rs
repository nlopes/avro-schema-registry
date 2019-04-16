#![feature(custom_attribute)]
#![feature(result_map_or_else)]
#![feature(trait_alias)]
#![feature(type_ascription)]

extern crate actix;
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate json;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod api;
pub mod app;
pub mod db;
pub mod health;
pub mod middleware;
