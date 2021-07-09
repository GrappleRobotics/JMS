table! {
    use diesel::sql_types::*;
    use crate::models::*;

    matches (id) {
        id -> Int4,
        match_type -> Match_type,
        set_number -> Int4,
        match_number -> Int4,
        red_teams -> Array<Int4>,
        blue_teams -> Array<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::*;

    teams (id) {
        id -> Int4,
        name -> Varchar,
        affiliation -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        notes -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    matches,
    teams,
);
