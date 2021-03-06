use std::fs::{self};
use std::path::Path;

use crate::api::item_management::models::Item;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use crate::settings::Settings;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::State;

#[delete("/item/<iid>")]
pub(crate) async fn delete_item(
    user: UserLoggedIn,
    iid: i32,
    conn: DbConn,
    settings: &State<Settings>,
) -> Result<(), ErrorResponse> {
    let item_list = conn
        .run(move |c| {
            use schema::items::dsl::*;
            items
                .filter(user_id.eq(user.0.id).and(id.eq(iid)))
                .load::<Item>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load item".to_string())
                })
        })
        .await?;

    let cid = item_list.first().map(|c| c.id).ok_or_else(|| {
        ErrorResponse::new(Status { code: 404 }, "Couldn't load item".to_string())
    })?;

    let image_file = Path::new(&settings.image_folder).join(cid.to_string());
    conn.run(move |c| {
        c.build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|| {
                {
                    use schema::uses::dsl::*;
                    diesel::delete(uses.filter(item_id.eq(cid))).execute(c)
                }?;
                {
                    use schema::item_inventory::dsl::*;
                    diesel::delete(item_inventory.filter(item_id.eq(cid))).execute(c)
                }?;
                {
                    use schema::items::dsl::*;
                    diesel::delete(items.filter(id.eq(iid))).execute(c)
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
