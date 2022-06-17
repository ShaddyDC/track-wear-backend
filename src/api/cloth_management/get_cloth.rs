use std::fs::File;
use std::path::Path;

use crate::api::cloth_management::models::Cloth;
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
pub struct ClothOut {
    pub id: i32,
    pub user_id: i32,
    pub cloth_name: String,
    pub count: i32,
}

#[get("/cloth/<cloth>")]
pub(crate) async fn get_cloth(
    user: UserLoggedIn,
    cloth: i32,
    conn: DbConn,
) -> Result<Json<ClothOut>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let cloth_list = conn
        .run(move |c| {
            clothes
                .filter(
                    user_id
                        .eq(user.0.id)
                        .and(schema::clothes::columns::id.eq(cloth)),
                )
                .load::<Cloth>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string())
                })
        })
        .await?;

    let cloth = cloth_list
        .first()
        .map(|cloth| ClothOut {
            id: cloth.id,
            user_id: cloth.user_id,
            cloth_name: cloth.cloth_name.clone(),
            count: 0,
        })
        .ok_or_else(|| {
            ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
        })?;

    use schema::wears::dsl::*;

    let count = conn
        .run(move |c| {
            wears
                .filter(cloth_id.eq(cloth.id))
                .count()
                .get_result(c)
                .map(|val: i64| val as i32)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 404 }, "Couldn't get wear count".to_string())
                })
        })
        .await?;

    Ok(Json(ClothOut { count, ..cloth }))
}

#[get("/cloth/<cloth>/image")]
pub(crate) async fn get_cloth_image(
    user: UserLoggedIn,
    cloth: i32,
    settings: &State<Settings>,
    conn: DbConn,
) -> Result<Option<File>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let image = conn
        .run(move |c| {
            clothes
                .filter(user_id.eq(user.0.id).and(id.eq(cloth)))
                .limit(1)
                .load::<Cloth>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't access database".to_string())
                })
        })
        .await?
        .first()
        .and_then(|_| {
            let image_file = Path::new(&settings.image_folder).join(cloth.to_string());
            File::open(&image_file).ok()
        });

    Ok(image)
}
