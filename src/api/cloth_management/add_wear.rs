use crate::api::cloth_management::models::Cloth;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::schema::wears;
use diesel::prelude::*;
use rocket::http::Status;

#[derive(Insertable, AsChangeset)]
#[table_name = "wears"]
struct NewWear {
    cloth_id: i32,
}

#[post("/cloth/<cloth>/add_wear")]
pub(crate) async fn add_wear(
    cloth: i32,
    user: UserLoggedIn,
    conn: DbConn,
) -> Result<(), ErrorResponse> {
    use schema::clothes::dsl::*;

    let cloth_list = conn
        .run(move |c| {
            clothes
                .filter(user_id.eq(user.0.id))
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
