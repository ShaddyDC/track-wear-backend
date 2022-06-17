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

joinable!(items -> users (user_id));
joinable!(uses -> items (item_id));

allow_tables_to_appear_in_same_query!(items, users, uses,);
