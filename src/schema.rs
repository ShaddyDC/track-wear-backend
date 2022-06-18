table! {
    item_inventory (id) {
        id -> Int4,
        movement -> Int4,
        item_id -> Int4,
        update_time -> Timestamp,
    }
}

table! {
    item_tags (id) {
        id -> Int4,
        item_id -> Int4,
        tag_id -> Int4,
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
    tags (id) {
        id -> Int4,
        user_id -> Int4,
        tag_name -> Varchar,
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

joinable!(item_tags -> tags (tag_id));
joinable!(uses -> items (item_id));

allow_tables_to_appear_in_same_query!(
    item_inventory,
    item_tags,
    items,
    tags,
    users,
    uses,
);
