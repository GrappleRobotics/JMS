table! {
    event_details (id) {
        id -> Integer,
        code -> Nullable<Text>,
        event_name -> Nullable<Text>,
    }
}

table! {
    matches (id) {
        id -> Integer,
        match_type -> Text,
        set_number -> Integer,
        match_number -> Integer,
        red_teams -> Text,
        blue_teams -> Text,
    }
}

table! {
    schedule_blocks (id) {
        id -> Integer,
        name -> Text,
        start_time -> Timestamp,
        end_time -> Timestamp,
        cycle_time -> BigInt,
        quals -> Bool,
    }
}

table! {
    teams (id) {
        id -> Integer,
        name -> Nullable<Text>,
        affiliation -> Nullable<Text>,
        location -> Nullable<Text>,
        notes -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    event_details,
    matches,
    schedule_blocks,
    teams,
);
