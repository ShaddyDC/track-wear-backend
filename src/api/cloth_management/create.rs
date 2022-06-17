use std::path::Path;

use crate::api::cloth_management::get_cloth::ClothOut;
use crate::api::cloth_management::models::Cloth;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::clothes;
use crate::settings::Settings;
use diesel::prelude::*;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[derive(FromForm)]
pub struct FormCloth<'a> {
    name: String,
    image: TempFile<'a>,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "clothes"]
struct NewCloth {
    cloth_name: String,
    user_id: i32,
}

#[post("/create_cloth", data = "<form_cloth>")]
pub(crate) async fn create_cloth(
    mut form_cloth: Form<FormCloth<'_>>,
    user: UserLoggedIn,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<Json<ClothOut>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let new_cloth = NewCloth {
        cloth_name: form_cloth.name.clone(),
        user_id: user.0.id,
    };

    let cloth = conn
        .run(move |c| {
            diesel::insert_into(clothes)
                .values(&new_cloth)
                .get_result::<Cloth>(c)
                .map_err(|err| {
                    ErrorResponse::new(
                        Status { code: 500 },
                        format!("Couldn't update cloth: {}", err),
                    )
                })
        })
        .await?;

    let image_file = Path::new(&settings.image_folder).join(cloth.id.to_string());
    if let Err(err) = form_cloth.image.copy_to(image_file).await {
        // roll back db
        // TODO Wrap this in transaction. As file copy is async, only possible when diesel is async
        // https://github.com/diesel-rs/diesel/issues/399
        conn.run(move |c| diesel::delete(&cloth).execute(c))
            .await
            .ok();

        return Err(ErrorResponse::new(
            Status { code: 500 },
            format!("Couldn't save image: {}", err),
        ));
    }

    Ok(Json(ClothOut {
        id: cloth.id,
        user_id: cloth.user_id,
        cloth_name: cloth.cloth_name,
        count: 0,
    }))
}
