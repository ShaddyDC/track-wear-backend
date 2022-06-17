use crate::api::item_management::get_item::ItemOut;
use crate::api::item_management::models::Item;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/items")]
pub(crate) async fn get_items(
    user: UserLoggedIn,
    conn: DbConn,
) -> Result<Json<Vec<ItemOut>>, ErrorResponse> {
    use schema::items::dsl::*;

    // Switch when diesel 2 is supported in Rocket
    // https://github.com/SergioBenitez/Rocket/issues/2209
    // Also use opportunity to use .first() in diesel instead of vector

    // use crate::schema::*;
    // use diesel::dsl::count;
    //
    // let item_list = conn
    //     .run(move |c| {
    //         items::table
    //             .filter(itemes::user_id.eq(user.id))
    //             .left_join(uses::table)
    //             .group_by(items::id)
    //             .select((
    //                 (items::id, items::user_id, items::item_name),
    //                 count(uses::id),
    //             ))
    //             .load::<(item, i64)>(c)
    //             .map_err(|_| {
    //                 ErrorResponse::new(Status { code: 500 }, "Couldn't load items".to_string())
    //             })
    //     })
    //     .await;

    let item_list = conn
        .run(move |c| {
            items
                .filter(user_id.eq(user.0.id))
                .load::<Item>(c)
                .map_err(|_| {
                    ErrorResponse::new(Status { code: 500 }, "Couldn't load items".to_string())
                })
        })
        .await?;

    use schema::uses::dsl::*;

    let out = conn
        .run(|c| {
            item_list
                .into_iter()
                .map(|item| {
                    let count = uses
                        .filter(item_id.eq(item.id))
                        .count()
                        .get_result(c)
                        .map(|val: i64| val as i32)
                        .map_err(|_| {
                            ErrorResponse::new(
                                Status { code: 404 },
                                "Couldn't get wear count".to_string(),
                            )
                        })?;

                    Ok(ItemOut {
                        id: item.id,
                        user_id: item.user_id,
                        item_name: item.item_name,
                        count,
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .await?;

    Ok(Json(out))
}
