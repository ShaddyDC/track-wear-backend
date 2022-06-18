use crate::api::user_management::models::UserLoggedIn;
use crate::db::DbConn;
use crate::error::ErrorResponse;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;

#[delete("/tag/<tid>")]
pub(crate) async fn delete_tag(
    user: UserLoggedIn,
    tid: i32,
    conn: DbConn,
) -> Result<(), ErrorResponse> {
    conn.run(move |c| {
        c.build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|| {
                {
                    // Tag owned
                    use schema::tags::dsl::*;
                    tags.filter(id.eq(tid).and(user_id.eq(user.0.id)))
                        .select(id)
                        .first::<i32>(c)
                }?;
                {
                    use schema::item_tags::dsl::*;
                    diesel::delete(item_tags.filter(tag_id.eq(tid))).execute(c)
                }?;
                {
                    use schema::tags::dsl::*;
                    diesel::delete(tags.filter(id.eq(tid))).execute(c)
                }?;

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
