use rocket::http::Status;

#[get("/health")]
pub fn health() -> Status {
  Status::Ok
}