use std::fmt::Debug;
use std::fs::{self, File};
use std::path::Path;

use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::clothes;
use crate::schema::wears;
use crate::settings::Settings;
use crate::user_management::UserOut;
use diesel::prelude::*;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

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

#[derive(Queryable, Debug)]
pub struct Cloth {
    pub id: i32,
    pub user_id: i32,
    pub cloth_name: String,
}

#[derive(Serialize)]
pub struct ClothOut {
    pub id: i32,
    pub user_id: i32,
    pub cloth_name: String,
    pub count: i32,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "wears"]
struct NewWear {
    cloth_id: i32,
}

#[post("/create_cloth", data = "<form_cloth>")]
pub(crate) async fn create_cloth(
    mut form_cloth: Form<FormCloth<'_>>,
    user: UserOut,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<Json<ClothOut>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let new_cloth = NewCloth {
        cloth_name: form_cloth.name.clone(),
        user_id: user.id,
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

    form_cloth
        .image
        .persist_to(image_file)
        .await
        .map_err(|err| {
            ErrorResponse::new(
                Status { code: 500 },
                format!("Couldn't save image: {}", err),
            )
        })?;

    Ok(Json(ClothOut {
        id: cloth.id,
        user_id: cloth.user_id,
        cloth_name: cloth.cloth_name,
        count: 0,
    }))
}

#[get("/clothes")]
pub(crate) async fn get_clothes(
    user: UserOut,
    conn: DbConn,
) -> Result<Json<Vec<ClothOut>>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let cloth_list = conn
        .run(move |c| {
            clothes
                .filter(user_id.eq(user.id))
                .load::<Cloth>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load clothes".to_string())
                })
        })
        .await?;

    use schema::wears::dsl::*;

    let out = conn
        .run(|c| {
            cloth_list
                .into_iter()
                .map(|cloth| {
                    let count = wears
                        .filter(cloth_id.eq(cloth.id))
                        .count()
                        .get_result(c)
                        .map(|val: i64| val as i32)
                        .map_err(|_| {
                            ErrorResponse::new(
                                Status { code: 404 },
                                "Couldn't get wear count".to_string(),
                            )
                        })?;

                    Ok(ClothOut {
                        id: cloth.id,
                        user_id: cloth.user_id,
                        cloth_name: cloth.cloth_name,
                        count,
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .await?;

    Ok(Json(out))
}

#[get("/cloth/<cloth>")]
pub(crate) async fn get_cloth(
    user: UserOut,
    cloth: i32,
    conn: DbConn,
) -> Result<Json<ClothOut>, ErrorResponse> {
    use schema::clothes::dsl::*;

    let cloth_list = conn
        .run(move |c| {
            clothes
                .filter(
                    user_id
                        .eq(user.id)
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

#[delete("/cloth/<cloth_id>")]
pub(crate) async fn delete_cloth(
    user: UserOut,
    cloth_id: i32,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<(), ErrorResponse> {
    use schema::clothes::dsl::*;

    let cloth_list = conn
        .run(move |c| {
            clothes
                .filter(user_id.eq(user.id).and(id.eq(cloth_id)))
                .load::<Cloth>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string())
                })
        })
        .await?;

    let cloth_id = cloth_list.first().map(|c| c.id).ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
    })?;

    let image_file = Path::new(&settings.image_folder).join(cloth_id.to_string());
    fs::remove_file(image_file).ok();

    conn.run(move |c| {
        diesel::delete(clothes.filter(id.eq(cloth_id)))
            .execute(c)
            .map_err(|_| {
                ErrorResponse::new(
                    Status { code: 500 },
                    "Couldn't delete database intry".to_string(),
                )
            })
    })
    .await?;

    Ok(())
}

#[get("/cloth/<cloth_id>/image")]
pub(crate) async fn get_cloth_image(
    _name: UserOut,
    cloth_id: u32,
    settings: &State<Settings>,
) -> Option<File> {
    let image_file = Path::new(&settings.image_folder).join(cloth_id.to_string());
    File::open(&image_file).ok()
}

#[post("/cloth/<cloth>/add_wear")]
pub(crate) async fn add_wear(cloth: i32, user: UserOut, conn: DbConn) -> Result<(), ErrorResponse> {
    use schema::clothes::dsl::*;

    let cloth_list = conn
        .run(move |c| {
            clothes
                .filter(user_id.eq(user.id))
                .filter(schema::clothes::columns::id.eq(cloth))
                .load::<Cloth>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string())
                })
        })
        .await?;

    cloth_list.first().ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
    })?;

    use schema::wears::dsl::*;

    let wear = NewWear { cloth_id: cloth };

    conn.run(move |c| {
        diesel::insert_into(wears)
            .values(&wear)
            .execute(c)
            .map_err(|err| {
                ErrorResponse::new(
                    Status { code: 500 },
                    format!("Couldn't update wear: {}", err),
                )
            })
    })
    .await?;

    Ok(())
}
