use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/item/<item>/tags")]
pub(crate) async fn get_item_tags(
    user: UserLoggedIn,
    item: i32,
    conn: DbConn,
) -> Result<Json<Vec<String>>, ErrorResponse> {
    use schema::item_tags;
    use schema::items;
    use schema::tags;
    use schema::users;

    let out = conn
        .run(move |c| {
            users::table
                .filter(users::id.eq(user.0.id))
                .inner_join(items::table.on(items::user_id.eq(users::id)))
                .filter(items::id.eq(item))
                .inner_join(item_tags::table.on(item_tags::item_id.eq(items::id)))
                .inner_join(tags::table.on(tags::id.eq(item_tags::item_id)))
                .select(tags::tag_name)
                .load::<String>(c)
        })
        .await
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, format!("Couldn't get tags")))?;

    Ok(Json(out))
}
