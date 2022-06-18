use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema::{self, item_tags};
use diesel::prelude::*;
use rocket::form::Form;
use rocket::http::Status;

#[derive(Insertable, AsChangeset)]
#[table_name = "item_tags"]
struct NewItemTag {
    item_id: i32,
    tag_id: i32,
}

#[derive(FromForm)]
pub struct FormTag {
    tag_id: i32,
}

#[post("/item/<item>/remove_tag", data = "<form_tag>")]
pub(crate) async fn remove_tag(
    item: i32,
    user: UserLoggedIn,
    conn: DbConn,
    form_tag: Form<FormTag>,
) -> Result<(), ErrorResponse> {
    conn.run::<_, Result<(), diesel::result::Error>>(move |c| {
        use schema::item_tags;
        use schema::tags;
        // TODO Probably better to use a join here in the future once supported by diesel
        // https://github.com/diesel-rs/diesel/issues/1478
        diesel::delete(item_tags::table)
            .filter(item_tags::item_id.eq(item))
            .filter(
                item_tags::tag_id.eq_any(
                    tags::table
                        .filter(tags::user_id.eq(user.0.id))
                        .filter(tags::id.eq(form_tag.tag_id))
                        .select(tags::id),
                ),
            )
            .execute(c)
            .map(|_| (()))
    })
    .await
    .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't remove tag".to_string()))?;

    Ok(())
}
