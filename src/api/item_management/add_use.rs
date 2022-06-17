use crate::api::item_management::models::Item;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::uses;
use diesel::prelude::*;
use rocket::http::Status;

#[derive(Insertable, AsChangeset)]
#[table_name = "uses"]
struct NewUse {
    item_id: i32,
}

#[post("/item/<item>/add_use")]
pub(crate) async fn add_use(
    item: i32,
    user: UserLoggedIn,
    conn: DbConn,
) -> Result<(), ErrorResponse> {
    use schema::items::dsl::*;

    let item_list = conn
        .run(move |c| {
            items
                .filter(user_id.eq(user.0.id))
                .filter(schema::items::columns::id.eq(item))
                .load::<Item>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load item".to_string())
                })
        })
        .await?;

    item_list.first().ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load item".to_string())
    })?;

    use schema::uses::dsl::*;

    let us = NewUse { item_id: item };

    conn.run(move |c| {
        diesel::insert_into(uses)
            .values(&us)
            .execute(c)
            .map_err(|err| {
                ErrorResponse::new(
                    Status { code: 500 },
                    format!("Couldn't update use: {}", err),
                )
            })
    })
    .await?;

    Ok(())
}
