use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::tags;
use diesel::prelude::*;
use rocket::form::Form;
use rocket::http::Status;

#[derive(Insertable, AsChangeset)]
#[table_name = "tags"]
struct NewTag {
    tag_name: String,
    user_id: i32,
}

#[derive(FromForm)]
pub struct FormTag {
    tag_name: String,
}

#[post("/tags/create", data = "<form_tag>")]
pub(crate) async fn create_tag(
    user: UserLoggedIn,
    conn: DbConn,
    form_tag: Form<FormTag>,
) -> Result<(), ErrorResponse> {
    use schema::tags::dsl::*;

    let tag = NewTag {
        tag_name: form_tag.into_inner().tag_name,
        user_id: user.0.id,
    };

    conn.run(move |c| {
        diesel::insert_into(tags)
            .values(&tag)
            .execute(c)
            .map_err(|err| {
                ErrorResponse::new(
                    Status { code: 500 },
                    format!("Couldn't create tag: {}", err),
                )
            })
    })
    .await?;

    Ok(())
}
