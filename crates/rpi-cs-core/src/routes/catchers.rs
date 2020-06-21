use rocket::http::Status;
use rocket::request::Request;
use rocket_contrib::json::Json;

#[derive(Serialize)]
pub struct ErrorMessage {
    pub message: String
}

#[catch(404)]
pub fn not_found(_req: &Request) -> Result<Json<ErrorMessage>, Status> {
    Ok(Json(ErrorMessage {
        message: "The requested resource was not found.".to_string(),
    }))
}

#[catch(500)]
pub fn server_error(_req: &Request) -> Result<Json<ErrorMessage>, Status> {
    Ok(Json(ErrorMessage {
        message: "The server encountered an internal error while processing the request"
            .to_string(),
    }))
}
