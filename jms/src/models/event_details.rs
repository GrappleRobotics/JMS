use diesel::{ExpressionMethods, QueryResult, RunQueryDsl};

use crate::{db, schema::event_details};

#[derive(Insertable, Queryable, Debug, Clone, AsChangeset, serde::Serialize, serde::Deserialize)]
#[table_name = "event_details"]
#[changeset_options(treat_none_as_null = "true")]
pub struct EventDetails {
  pub id: i32,
  pub code: Option<String>,
  pub event_name: Option<String>,
}

impl EventDetails {
  pub fn get(conn: &db::ConnectionT) -> QueryResult<EventDetails> {
    use crate::schema::event_details::dsl::*;

    let record = event_details.first::<EventDetails>(conn);
    match record {
      Ok(details) => Ok(details),
      Err(diesel::NotFound) => {
        // Insert the default
        diesel::insert_into(event_details)
          .values((code.eq::<Option<String>>(None), event_name.eq::<Option<String>>(None)))
          .execute(conn)?;

        event_details.first::<EventDetails>(conn)
      }
      Err(e) => Err(e),
    }
  }
}
