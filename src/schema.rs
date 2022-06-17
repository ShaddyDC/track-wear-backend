table! {
    item_inventory (id) {
        id -> Int4,
        movement -> Int4,
        item_id -> Int4,
        update_time -> Timestamp,
    }
}

table! {
    items (id) {
        id -> Int4,
        user_id -> Int4,
        item_name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        sub -> Varchar,
        username -> Varchar,
        email -> Varchar,
    }
}

table! {
    uses (id) {
        id -> Int4,
        item_id -> Int4,
        date -> Date,
    }
}

joinable!(item_inventory -> items (item_id));
joinable!(items -> users (user_id));
joinable!(uses -> items (item_id));

allow_tables_to_appear_in_same_query!(
    item_inventory,
    items,
    users,
    uses,
);
