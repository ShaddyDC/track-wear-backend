table! {
    clothes (id) {
        id -> Int4,
        user_id -> Int4,
        cloth_name -> Varchar,
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
    wears (id) {
        id -> Int4,
        cloth_id -> Int4,
        date -> Date,
    }
}

joinable!(clothes -> users (user_id));
joinable!(wears -> clothes (cloth_id));

allow_tables_to_appear_in_same_query!(
    clothes,
    users,
    wears,
);
