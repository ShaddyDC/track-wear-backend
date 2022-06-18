use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/tags")]
pub(crate) async fn get_tags(
    user: UserLoggedIn,
    conn: DbConn,
) -> Result<Json<Vec<String>>, ErrorResponse> {
    use schema::tags;
    use schema::users;

    let out = conn
        .run(move |c| {
            users::table
                .filter(users::id.eq(user.0.id))
                .inner_join(tags::table.on(tags::user_id.eq(users::id)))
                .select(tags::tag_name)
                .load::<String>(c)
        })
        .await
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, format!("Couldn't get tags")))?;

    Ok(Json(out))
}
