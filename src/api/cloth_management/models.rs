use crate::schema::clothes;
use std::fmt::Debug;

#[derive(Queryable, Debug, Identifiable, AsChangeset)]
#[table_name = "clothes"]
pub struct Cloth {
    pub id: i32,
    pub user_id: i32,
    pub cloth_name: String,
}
