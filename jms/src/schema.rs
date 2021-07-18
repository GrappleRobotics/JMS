table! {
    event_details (id) {
        id -> Integer,
        code -> Nullable<Text>,
        event_name -> Nullable<Text>,
    }
}

table! {
    match_generation_records (id) {
        id -> Integer,
        team_balance -> Double,
        station_balance -> Double,
        cooccurrence -> Text,
        station_dist -> Text,
    }
}

table! {
    matches (id) {
        id -> Integer,
        start_time -> BigInt,
        match_type -> Text,
        set_number -> Integer,
        match_number -> Integer,
        red_teams -> Text,
        blue_teams -> Text,
        played -> Bool,
    }
}

table! {
    schedule_blocks (id) {
        id -> Integer,
        name -> Text,
        start_time -> BigInt,
        end_time -> BigInt,
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
    match_generation_records,
    matches,
    schedule_blocks,
    teams,
);
