table! {
    sites (id) {
        id -> Int4,
        name -> Varchar,
        zone -> Varchar,
        disabled -> Bool,
    }
}

table! {
    user_site_privileges (id) {
        id -> Int4,
        user_id -> Int4,
        site_id -> Int4,
        pattern -> Varchar,
        superuser -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        key -> Varchar,
        disabled -> Bool,
    }
}

joinable!(user_site_privileges -> sites (site_id));
joinable!(user_site_privileges -> users (user_id));

allow_tables_to_appear_in_same_query!(
    sites,
    user_site_privileges,
    users,
);
