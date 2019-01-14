table! {
    sites (id) {
        id -> Int4,
        name -> Varchar,
        zone -> Varchar,
        disabled -> Bool,
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

table! {
    user_site_privileges (id) {
        id -> Int4,
        user_id -> Int4,
        site_id -> Int4,
        pattern -> Nullable<Varchar>,
        superuser -> Bool,
    }
}

joinable!(user_site_privileges -> sites (site_id));
joinable!(user_site_privileges -> users (user_id));

allow_tables_to_appear_in_same_query!(
    sites,
    users,
    user_site_privileges,
);
