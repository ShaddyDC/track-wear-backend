use rocket::response::{Responder, Response};
use rocket::{
    http::{ContentType, Status},
    response,
    serde::json::Json,
    Request,
};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ApiError<'a> {
    err: &'a str,
}

pub(crate) struct ErrorResponse<T> {
    json: Json<T>,
    status: Status,
}

pub(crate) type ApiErrorResponse<'a> = ErrorResponse<ApiError<'a>>;

impl ErrorResponse<ApiError<'_>> {
    pub(crate) fn new(status: Status, err: &str) -> ErrorResponse<ApiError> {
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
