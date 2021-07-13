table! {
    use diesel::sql_types::*;
    use crate::models::*;

    matches (id) {
        id -> Nullable<Integer>,
        match_type -> Text,
        set_number -> Integer,
        match_number -> Integer,
        red_teams -> Text,
        blue_teams -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    teams (id) {
        id -> Nullable<Integer>,
        name -> Text,
        affiliation -> Nullable<Text>,
        location -> Nullable<Text>,
        notes -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    matches,
    teams,
);
