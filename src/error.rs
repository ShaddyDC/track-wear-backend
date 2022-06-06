use rocket::response::{Responder, Response};
use rocket::{
    http::{ContentType, Status},
    response,
    serde::json::Json,
    Request,
};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ApiError {
    err: String,
}

impl ApiError {
    pub(crate) fn new(err: String) -> ApiError {
        ApiError { err }
    }
}

#[derive(Debug)]
pub(crate) struct ErrorResponse<T = ApiError> {
    json: Json<T>,
    status: Status,
}

impl ErrorResponse<ApiError> {
    pub(crate) fn new(status: Status, err: String) -> ErrorResponse<ApiError> {
        ErrorResponse {
            json: Json(ApiError { err }),
            status,
        }
    }
}

impl<'r, T: serde::Serialize> Responder<'r, 'r> for ErrorResponse<T> {
    fn respond_to(self, req: &'r Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
