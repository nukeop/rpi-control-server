#![warn(clippy::all)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;
extern crate rppal;
extern crate serde_json;

pub mod api;
pub mod bme280;
pub mod routes;
pub mod weather;