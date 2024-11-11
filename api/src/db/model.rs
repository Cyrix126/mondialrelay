// Generated by diesel_ext

#![allow(unused)]
#![allow(clippy::all)]

use chrono::offset::Utc;
use chrono::DateTime;
use diesel::{
    prelude::{AsChangeset, Associations, Identifiable, Insertable},
    Queryable, Selectable,
};
#[derive(Queryable, Debug, Selectable, Insertable, Identifiable, PartialEq, Default)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = crate::db::schema::shipments)]
pub struct Shipment {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub order_id: i32,
    pub label_url: String,
    #[diesel(skip_insertion)]
    #[diesel(deserialize_as = DateTime<Utc>)]
    pub created_at: Option<DateTime<Utc>>,
}
