use rocket;

use crate::routes::{catchers, health};

pub fn start() -> rocket::Rocket {
  rocket::ignite()
    .register(catchers![catchers::not_found, catchers::server_error])
    .mount("/api", routes![health::health])
}
