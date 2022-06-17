use std::fs::{self};
use std::path::Path;

use crate::api::cloth_management::models::Cloth;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::settings::Settings;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::State;

#[delete("/cloth/<cid>")]
pub(crate) async fn delete_cloth(
    user: UserLoggedIn,
    cid: i32,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<(), ErrorResponse> {
    let cloth_list = conn
        .run(move |c| {
            use schema::clothes::dsl::*;
            clothes
                .filter(user_id.eq(user.0.id).and(id.eq(cid)))
                .load::<Cloth>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load cloth".to_string())
                })
        })
        .await?;

    let cid = cloth_list.first().map(|c| c.id).ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load cloth".to_string())
    })?;

    let image_file = Path::new(&settings.image_folder).join(cid.to_string());
    conn.run(move |c| {
        c.build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|| {
                {
                    use schema::wears::dsl::*;
                    diesel::delete(wears.filter(cloth_id.eq(cid))).execute(c)
                }?;
                {
                    use schema::clothes::dsl::*;
                    diesel::delete(clothes.filter(id.eq(cid))).execute(c)
                }?;

                fs::remove_file(image_file).ok();

                Ok(())
            })
    })
    .await
    .map_err(|_| {
        ErrorResponse::new(
            Status { code: 500 },
            "Couldn't delete database entries".to_string(),
        )
    })?;

    Ok(())
}
