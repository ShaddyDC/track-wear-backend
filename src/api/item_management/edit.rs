use std::path::Path;

use crate::api::item_management::get_item::ItemOut;
use crate::api::item_management::models::Item;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::settings::Settings;
use diesel::prelude::*;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[derive(FromForm)]
pub struct FormEditItem<'a> {
    name: Option<String>,
    image: Option<TempFile<'a>>,
}

#[post("/item/<item_id>/edit", data = "<form_item>")]
pub(crate) async fn edit_item(
    mut form_item: Form<FormEditItem<'_>>,
    item_id: i32,
    user: UserLoggedIn,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<Json<ItemOut>, ErrorResponse> {
    use schema::items::dsl::*;

    let mut item = conn
        .run(move |c| {
            items
                .filter(user_id.eq(user.0.id).and(id.eq(item_id)))
                .first::<Item>(c)
        })
        .await
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't get item".to_string()))?;
    let uid = item.user_id;
    let final_name = form_item.name.as_ref().unwrap_or(&item.item_name).clone();

    if let Some(name) = &form_item.name {
        item.item_name = name.clone();

        conn.run(move |c| item.save_changes::<Item>(c))
            .await
            .map_err(|err| {
                ErrorResponse::new(
                    Status { code: 500 },
                    format!("Couldn't update data: {}", err),
                )
            })?;
    };

    if let Some(file) = &mut form_item.image {
        let image_file = Path::new(&settings.image_folder).join(item_id.to_string());
        file.copy_to(image_file).await.map_err(|err| {
            ErrorResponse::new(
                Status { code: 500 },
                format!("Couldn't save image: {}", err),
            )
        })?;
    }

    Ok(Json(ItemOut {
        id: item_id,
        user_id: uid,
        item_name: final_name,
        count: 0,
    }))
}
