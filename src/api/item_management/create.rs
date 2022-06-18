use std::path::Path;

use crate::api::item_management::get_item::ItemOut;
use crate::api::item_management::models::Item;
use crate::api::item_management::modify_inventory::NewInventory;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::items;
use crate::settings::Settings;
use diesel::prelude::*;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[derive(FromForm)]
pub struct FormItem<'a> {
    name: String,
    image: TempFile<'a>,
    count: Option<i32>,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "items"]
struct NewItem {
    item_name: String,
    user_id: i32,
}

#[post("/create_item", data = "<form_item>")]
pub(crate) async fn create_item(
    mut form_item: Form<FormItem<'_>>,
    user: UserLoggedIn,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<Json<ItemOut>, ErrorResponse> {
    use schema::items::dsl::*;

    let new_item = NewItem {
        item_name: form_item.name.clone(),
        user_id: user.0.id,
    };
    let movement = form_item.count.unwrap_or(1);

    let item = conn
        .run(move |c| {
            c.transaction::<_, diesel::result::Error, _>(|| {
                let item = diesel::insert_into(items)
                    .values(&new_item)
                    .get_result::<Item>(c)?;

                let new_inventory = NewInventory {
                    item_id: item.id,
                    movement,
                };

                diesel::insert_into(schema::item_inventory::dsl::item_inventory)
                    .values(&new_inventory)
                    .execute(c)?;

                Ok(item)
            })
        })
        .await
        .map_err(|err| {
            ErrorResponse::new(
                Status { code: 500 },
                format!("Couldn't update item: {}", err),
            )
        })?;

    let image_file = Path::new(&settings.image_folder).join(item.id.to_string());
    if let Err(err) = form_item.image.copy_to(image_file).await {
        // roll back db
        // TODO Wrap this in transaction. As file copy is async, only possible when diesel is async
        // https://github.com/diesel-rs/diesel/issues/399
        conn.run(move |c| {
            diesel::delete(schema::item_inventory::table)
                .filter(schema::item_inventory::item_id.eq(item.id))
                .execute(c)
                .and_then(|_| diesel::delete(&item).execute(c))
        })
        .await
        .ok();

        return Err(ErrorResponse::new(
            Status { code: 500 },
            format!("Couldn't save image: {}", err),
        ));
    }

    Ok(Json(ItemOut {
        id: item.id,
        user_id: item.user_id,
        item_name: item.item_name,
        count: 0,
    }))
}
