table! {
    teams (id) {
        id -> Int4,
        name -> Varchar,
        affiliation -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        notes -> Nullable<Varchar>,
    }
}
