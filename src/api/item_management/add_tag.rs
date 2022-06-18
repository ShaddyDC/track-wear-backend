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

#[post("/item/<item>/add_tag", data = "<form_tag>")]
pub(crate) async fn add_tag(
    item: i32,
    user: UserLoggedIn,
    conn: DbConn,
    form_tag: Form<FormTag>,
) -> Result<(), ErrorResponse> {
    conn.run(move |c| {
        use schema::item_tags;
        use schema::items;
        use schema::tags;
        use schema::users;

        let pair = users::table
            .filter(users::id.eq(user.0.id))
            .inner_join(items::table.on(items::user_id.eq(users::id)))
            .filter(items::id.eq(item))
            .inner_join(tags::table.on(tags::user_id.eq(users::id)))
            .filter(tags::id.eq(form_tag.tag_id))
            .select((items::id, tags::id));

        diesel::insert_into(item_tags::table)
            .values(pair)
            .into_columns((item_tags::item_id, item_tags::tag_id))
            .execute(c)
    })
    .await
    .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't add tag".to_string()))?;

    Ok(())
}
