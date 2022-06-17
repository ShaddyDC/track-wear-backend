use std::fs::File;
use std::path::Path;

use crate::api::item_management::models::Item;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::settings::Settings;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

#[derive(Serialize)]
pub struct ItemOut {
    pub id: i32,
    pub user_id: i32,
    pub item_name: String,
    pub count: i32,
}

#[get("/item/<item>")]
pub(crate) async fn get_item(
    user: UserLoggedIn,
    item: i32,
    conn: DbConn,
) -> Result<Json<ItemOut>, ErrorResponse> {
    use schema::items::dsl::*;

    let item_list = conn
        .run(move |c| {
            items
                .filter(
                    user_id
                        .eq(user.0.id)
                        .and(schema::items::columns::id.eq(item)),
                )
                .load::<Item>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load item".to_string())
                })
        })
        .await?;

    let item = item_list
        .first()
        .map(|first| ItemOut {
            id: first.id,
            user_id: first.user_id,
            item_name: first.item_name.clone(),
            count: 0,
        })
        .ok_or_else(|| {
            ErrorResponse::new(Status { code: 404 }, "Couldn't load item".to_string())
        })?;

    use schema::uses::dsl::*;

    let count = conn
        .run(move |c| {
            uses.filter(item_id.eq(item.id))
                .count()
                .get_result(c)
                .map(|val: i64| val as i32)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 404 }, "Couldn't get use count".to_string())
                })
        })
        .await?;

    Ok(Json(ItemOut { count, ..item }))
}

#[get("/item/<item>/image")]
pub(crate) async fn get_item_image(
    user: UserLoggedIn,
    item: i32,
    settings: &State<Settings>,
    conn: DbConn,
) -> Result<Option<File>, ErrorResponse> {
    use schema::items::dsl::*;

    let image = conn
        .run(move |c| {
            items
                .filter(user_id.eq(user.0.id).and(id.eq(item)))
                .limit(1)
                .load::<Item>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't access database".to_string())
                })
        })
        .await?
        .first()
        .and_then(|_| {
            let image_file = Path::new(&settings.image_folder).join(item.to_string());
            File::open(&image_file).ok()
        });

    Ok(image)
}
