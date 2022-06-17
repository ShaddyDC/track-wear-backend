use std::path::Path;

use crate::api::cloth_management::get_cloth::ClothOut;
use crate::api::cloth_management::models::Cloth;
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
pub struct FormEditCloth<'a> {
    name: Option<String>,
    image: Option<TempFile<'a>>,
}

#[post("/cloth/<cloth_id>/edit", data = "<form_cloth>")]
pub(crate) async fn edit_cloth(
    mut form_cloth: Form<FormEditCloth<'_>>,
    cloth_id: i32,
    user: UserLoggedIn,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<Json<ClothOut>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let mut cloth = conn
        .run(move |c| {
            clothes
                .filter(user_id.eq(user.0.id).and(id.eq(cloth_id)))
                .first::<Cloth>(c)
        })
        .await
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't get cloth".to_string()))?;
    let uid = cloth.user_id;
    let final_name = form_cloth
        .name
        .as_ref()
        .unwrap_or(&cloth.cloth_name)
        .clone();

    if let Some(name) = &form_cloth.name {
        cloth.cloth_name = name.clone();

        conn.run(move |c| cloth.save_changes::<Cloth>(c))
            .await
            .map_err(|err| {
                ErrorResponse::new(
                    Status { code: 500 },
                    format!("Couldn't update data: {}", err),
                )
            })?;
    };

    if let Some(file) = &mut form_cloth.image {
        let image_file = Path::new(&settings.image_folder).join(cloth_id.to_string());
        file.copy_to(image_file).await.map_err(|err| {
            ErrorResponse::new(
                Status { code: 500 },
                format!("Couldn't save image: {}", err),
            )
        })?;
    }

    Ok(Json(ClothOut {
        id: cloth_id,
        user_id: uid,
        cloth_name: final_name,
        count: 0,
    }))
}
