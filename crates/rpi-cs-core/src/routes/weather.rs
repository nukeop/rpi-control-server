use rocket::http::Status;

#[get("/weather")]
pub fn weather() -> Result<(), Status> {
  
}