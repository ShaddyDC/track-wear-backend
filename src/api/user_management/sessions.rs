use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use crate::api::user_management::login::SessionCookie;
use crate::api::user_management::models::User;
use crate::db::DbConn;
use crate::error::ApiError;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::outcome::{try_outcome, IntoOutcome};
use rocket::request::{self, FromRequest, Outcome};
use rocket::{Request, State};

use super::models::{UserLoggedIn, UserOut};

pub(crate) struct UserSession {
    pub(crate) sessions: Mutex<HashMap<String, String>>,
}

impl UserSession {
    pub(crate) fn new() -> UserSession {
        UserSession {
            sessions: Mutex::new(HashMap::<String, String>::new()),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserLoggedIn {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let session_cookie = try_outcome!(req
            .cookies()
            .get_private("session")
            .ok_or_else(|| ApiError::new("No session set".to_string()))
            .or_forward(()));

        let session_cookie_string = session_cookie.value();

        let session_cookie_value =
            try_outcome!(serde_json::from_str::<SessionCookie>(session_cookie_string)
                .map_err(|_| ApiError::new("Couldn't parse session".to_string()))
                .or_forward(()));

        let session_age = try_outcome!(session_cookie_value
            .creation_time
            .elapsed()
            .map_err(|_| ApiError::new("Couldn't determine session age".to_string()))
            .or_forward(()));

        let max_age = Duration::from_secs(60 * 60 * 24 * 30);
        if session_age > max_age {
            return request::Outcome::Failure((
                Status { code: 401 },
                ApiError::new("Session too old".to_string()),
            ));
        }

        let sessions = try_outcome!(req.guard::<&State<UserSession>>().await.map_failure(|_| {
            (
                Status { code: 500 },
                ApiError::new("Couldn't get UserSession".to_string()),
            )
        }));
        let user_id = {
            let sessions = try_outcome!(sessions
                .sessions
                .lock()
                .map_err(|_| ApiError::new("Couldn't get user sessions".to_string()))
                .or_forward(()));

            try_outcome!(sessions
                .get(&session_cookie_value.session_key)
                .ok_or_else(|| ApiError::new("No session found".to_string()))
                .or_forward(()))
            .clone()
        };

        let conn = try_outcome!(req.guard::<DbConn>().await.map_failure(|_| {
            (
                Status { code: 500 },
                ApiError::new("Couldn't get database connection".to_string()),
            )
        }));

        use schema::users::dsl::*;

        let user_vec = try_outcome!(
            conn.run(|c| users
                .filter(sub.eq(user_id))
                .load::<User>(c)
                .map_err(|_| ApiError::new("Couldn't load user from database".to_string()))
                .or_forward(()))
                .await
        );

        let user = try_outcome!(user_vec
            .first()
            .ok_or_else(|| { ApiError::new("User not in database".to_string()) })
            .or_forward(()));

        Outcome::Success(UserLoggedIn(UserOut {
            id: user.id,
            sub: user.sub.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
        }))
    }
}
