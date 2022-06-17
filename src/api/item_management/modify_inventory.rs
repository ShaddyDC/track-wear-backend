use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::item_inventory;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use rocket::form::Form;
use rocket::http::Status;

#[derive(FromForm)]
pub struct FormInventory {
    item_id: i32,
    movement: i32,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "item_inventory"]
pub(super) struct NewInventory {
    pub(super) item_id: i32,
    pub(super) movement: i32,
}

#[post("/modify_inventory", data = "<form_inventory>")]
pub(crate) async fn modify_inventory(
    form_inventory: Form<FormInventory>,
    user: UserLoggedIn,
    conn: DbConn,
) -> Result<&'static str, ErrorResponse> {
    use schema::items::dsl::*;

    conn.run(move |c| {
        diesel::insert_into(schema::item_inventory::dsl::item_inventory)
            .values(
                items
                    .filter(user_id.eq(user.0.id).and(id.eq(form_inventory.item_id)))
                    .limit(1)
                    .select((id, form_inventory.movement.into_sql::<Integer>())),
            )
            .into_columns((
                schema::item_inventory::columns::item_id,
                schema::item_inventory::columns::movement,
            ))
            .returning(schema::item_inventory::columns::id)
            .get_result::<i32>(c)
    })
    .await
    .map_err(|_| {
        ErrorResponse::new(Status { code: 500 }, "Error inserting movement".to_string())
    })?;

    Ok("Success")
}
