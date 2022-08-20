pub mod arena;
pub mod config;
pub mod db;
pub mod ds;
pub mod logging;
pub mod network;
pub mod reports;
pub mod schedule;
pub mod scoring;
pub mod ui;
pub mod utils;
pub mod electronics;
pub mod models;
pub mod tba;
pub mod imaging;
pub mod discord;

extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate rocket;

extern crate jms_macros;