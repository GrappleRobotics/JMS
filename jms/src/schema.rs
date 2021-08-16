table! {
    awards (id) {
        id -> Integer,
        name -> Text,
        recipients -> Text,
    }
}

table! {
    event_details (id) {
        id -> Integer,
        code -> Nullable<Text>,
        event_name -> Nullable<Text>,
    }
}

table! {
    match_generation_records (match_type) {
        match_type -> Text,
        data -> Nullable<Text>,
    }
}

table! {
    matches (id) {
        id -> Integer,
        start_time -> Nullable<BigInt>,
        match_type -> Text,
        set_number -> Integer,
        match_number -> Integer,
        blue_teams -> Text,
        red_teams -> Text,
        played -> Bool,
        score -> Nullable<Text>,
        winner -> Nullable<Text>,
        match_subtype -> Nullable<Text>,
        red_alliance -> Nullable<Integer>,
        blue_alliance -> Nullable<Integer>,
        score_time -> Nullable<BigInt>,
    }
}

table! {
    playoff_alliances (id) {
        id -> Integer,
        teams -> Text,
        ready -> Bool,
    }
}

table! {
    schedule_blocks (id) {
        id -> Integer,
        block_type -> Text,
        name -> Text,
        start_time -> BigInt,
        end_time -> BigInt,
        cycle_time -> BigInt,
    }
}

table! {
    team_rankings (team) {
        team -> Integer,
        rp -> Integer,
        auto_points -> Integer,
        endgame_points -> Integer,
        teleop_points -> Integer,
        random_num -> Integer,
        win -> Integer,
        loss -> Integer,
        tie -> Integer,
        played -> Integer,
    }
}

table! {
    teams (id) {
        id -> Integer,
        name -> Nullable<Text>,
        affiliation -> Nullable<Text>,
        location -> Nullable<Text>,
        notes -> Nullable<Text>,
        wpakey -> Nullable<Text>,
        schedule -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    awards,
    event_details,
    match_generation_records,
    matches,
    playoff_alliances,
    schedule_blocks,
    team_rankings,
    teams,
);
