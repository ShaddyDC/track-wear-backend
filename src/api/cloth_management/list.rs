use crate::api::cloth_management::get_cloth::ClothOut;
use crate::api::cloth_management::models::Cloth;
use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/clothes")]
pub(crate) async fn get_clothes(
    user: UserLoggedIn,
    conn: DbConn,
) -> Result<Json<Vec<ClothOut>>, ErrorResponse> {
    use schema::clothes::dsl::*;

    // Switch when diesel 2 is supported in Rocket
    // https://github.com/SergioBenitez/Rocket/issues/2209
    // Also use opportunity to use .first() in diesel instead of vector

    // use crate::schema::*;
    // use diesel::dsl::count;
    //
    // let cloth_list = conn
    //     .run(move |c| {
    //         clothes::table
    //             .filter(clothes::user_id.eq(user.id))
    //             .left_join(wears::table)
    //             .group_by(clothes::id)
    //             .select((
    //                 (clothes::id, clothes::user_id, clothes::cloth_name),
    //                 count(wears::id),
    //             ))
    //             .load::<(Cloth, i64)>(c)
    //             .map_err(|_| {
    //                 ErrorResponse::new(Status { code: 500 }, "Couldn't load clothes".to_string())
    //             })
    //     })
    //     .await;

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
