use rocket;
use rocket::Route;
use rocket::http::Method;

use crate::debug::DebugState;
use crate::env;
use crate::routes::{catchers, health, weather};

pub fn start() -> rocket::Rocket {
  let debug = env::get("DEBUG") == "1";

  rocket::ignite().register(catchers![catchers::not_found, catchers::server_error])
  .manage(DebugState{ debug })
  .mount("/api", routes![
    health::health,
    weather::weather
    ])
}
