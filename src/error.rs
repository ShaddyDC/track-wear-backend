use serde::Serialize;

#[derive(Serialize)]
struct ApiError {
    error: String,
}
