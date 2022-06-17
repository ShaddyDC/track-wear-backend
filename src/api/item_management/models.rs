use crate::schema::items;
use std::fmt::Debug;

#[derive(Queryable, Debug, Identifiable, AsChangeset)]
#[table_name = "items"]
pub struct Item {
    pub id: i32,
    pub user_id: i32,
    pub item_name: String,
}
