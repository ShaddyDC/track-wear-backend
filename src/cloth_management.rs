use std::fmt::Debug;
use std::fs::{self, File};

use crate::error::ErrorResponse;
use crate::schema::clothes;
use crate::schema::wears;
use crate::user_management::UserOut;
use crate::{db, schema};
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
    pool: &State<db::Pool>,
) -> Result<Json<ClothOut>, ErrorResponse> {
    let conn = pool.get().map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't connect to database".to_string(),
        )
    })?;

    use schema::clothes::dsl::*;

    let new_cloth = NewCloth {
        cloth_name: form_cloth.name.clone(),
        user_id: user.id,
    };

    let cloth = diesel::insert_into(clothes)
        .values(&new_cloth)
        .get_result::<Cloth>(&conn)
        .map_err(|err| {
            ErrorResponse::new(
                Status { code: 500 },
                format!("Couldn't update cloth: {}", err),
            )
        })?;

    form_cloth
        .image
        .persist_to(format!("runtime/images/{}", cloth.id))
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
    pool: &State<db::Pool>,
) -> Result<Json<Vec<ClothOut>>, ErrorResponse> {
    let conn = pool.get().map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't connect to database".to_string(),
        )
    })?;

    use schema::clothes::dsl::*;

    let list = clothes
        .filter(user_id.eq(user.id))
        .load::<Cloth>(&conn)
        .map_err(|_| {
            ErrorResponse::new(Status { code: 500 }, "Couldn't load clothes".to_string())
        })?;

    use schema::wears::dsl::*;

    let out = list
        .into_iter()
        .map(|cloth| {
            let count = wears
                .filter(cloth_id.eq(cloth.id))
                .count()
                .get_result(&conn)
                .map(|val: i64| val as i32)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 404 }, "Couldn't get wear count".to_string())
                })?;

            Ok(ClothOut {
                id: cloth.id,
                user_id: cloth.user_id,
                cloth_name: cloth.cloth_name,
                count,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(out))
}

#[get("/cloth/<cloth>")]
pub(crate) async fn get_cloth(
    user: UserOut,
    cloth: i32,
    pool: &State<db::Pool>,
) -> Result<Json<ClothOut>, ErrorResponse> {
    let conn = pool.get().map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't connect to database".to_string(),
        )
    })?;

    use schema::clothes::dsl::*;

    let list = clothes
        .filter(
            user_id
                .eq(user.id)
                .and(schema::clothes::columns::id.eq(cloth)),
        )
        .load::<Cloth>(&conn)
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string()))?;

    let cloth = list.first().ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
    })?;

    use schema::wears::dsl::*;

    let count = wears
        .filter(cloth_id.eq(cloth.id))
        .count()
        .get_result(&conn)
        .map(|val: i64| val as i32)
        .map_err(|_| {
            ErrorResponse::new(Status { code: 404 }, "Couldn't get wear count".to_string())
        })?;

    Ok(Json(ClothOut {
        id: cloth.id,
        user_id: cloth.user_id,
        cloth_name: cloth.cloth_name.clone(),
        count,
    }))
}

#[delete("/cloth/<cloth_id>")]
pub(crate) async fn delete_cloth(
    user: UserOut,
    cloth_id: i32,
    pool: &State<db::Pool>,
) -> Result<(), ErrorResponse> {
    let conn = pool.get().map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't connect to database".to_string(),
        )
    })?;

    use schema::clothes::dsl::*;

    let list = clothes
        .filter(user_id.eq(user.id).and(id.eq(cloth_id)))
        .load::<Cloth>(&conn)
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string()))?;

    let cloth = list.first().ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
    })?;

    let filename = format!("runtime/images/{}", cloth.id);
    match fs::remove_file(filename) {
        Ok(_) => {}
        Err(_) => {}
    };

    diesel::delete(clothes.filter(id.eq(cloth.id)))
        .execute(&conn)
        .map_err(|_| {
            ErrorResponse::new(
                Status { code: 500 },
                "Couldn't delete database intry".to_string(),
            )
        })?;

    Ok(())
}

#[get("/cloth/<cloth_id>/image")]
pub(crate) async fn get_cloth_image(_name: UserOut, cloth_id: u32) -> Option<File> {
    let filename = format!("runtime/images/{}", cloth_id);
    File::open(&filename).ok()
}

#[post("/cloth/<cloth>/add_wear")]
pub(crate) fn add_wear(
    cloth: i32,
    user: UserOut,
    pool: &State<db::Pool>,
) -> Result<(), ErrorResponse> {
    let conn = pool.get().map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't connect to database".to_string(),
        )
    })?;

    use schema::clothes::dsl::*;

    let list = clothes
        .filter(
            user_id
                .eq(user.id)
                .and(schema::clothes::columns::id.eq(cloth)),
        )
        .load::<Cloth>(&conn)
        .map_err(|_| ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string()))?;

    list.first().ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
    })?;

    use schema::wears::dsl::*;

    let wear = NewWear { cloth_id: cloth };

    diesel::insert_into(wears)
        .values(&wear)
        .execute(&conn)
        .map_err(|err| {
            ErrorResponse::new(
                Status { code: 500 },
                format!("Couldn't update wear: {}", err),
            )
        })?;

    Ok(())
}
