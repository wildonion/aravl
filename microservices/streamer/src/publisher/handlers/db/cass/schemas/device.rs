




/*  

TODO - 

    https://github.com/krojew/cdrs-tokio/blob/master/documentation
    https://github.com/krojew/cdrs-tokio/blob/master/cdrs-tokio/examples/crud_operations.rs
    https://github.com/krojew/cdrs-tokio/blob/master/cdrs-tokio/examples/multiple_thread.rs
    https://github.com/krojew/cdrs-tokio/blob/master/cdrs-tokio/examples/insert_collection.rs
    https://github.com/krojew/cdrs-tokio/blob/master/cdrs-tokio/examples/prepare_batch_execute.rs
    https://www.oreilly.com/library/view/cassandra-the-definitive/9781491933657/ch04.html

*/




use crate::handlers::db::cass::establish::CassSession;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;
use cdrs_tokio::query::*;
use cdrs_tokio::query_values;
use cdrs_tokio::frame::AsBytes;
use cdrs_tokio::types::from_cdrs::FromCdrsByName;
use cdrs_tokio::types::prelude::*;
use cdrs_tokio_helpers_derive::*;








/////////////////////////////////////////////////// CASSANDRA ARCHITECTURE ///////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////// https://github.com/krojew/cdrs-tokio/blob/master/type-mapping.md ////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
#[derive(Serialize, Deserialize, Copy, Clone, Debug, TryFromRow, TryFromUdt)]
pub struct GPSData{ //-- GPSData will be stored on the stack due to its static data types thus it's bounded to the Copy trait and we can take a reference from self
    pub id: Uuid,
    pub imei: i64,
    pub lat: i32,
    pub lon: i32,
    pub speed: f32,
    pub alt: i16,
    pub angle: i16,
    pub satellites: i8,
    pub devicetime: i64,
}






impl GPSData{
    pub async fn first(session: Arc<CassSession>) -> Self{
        GPSData{id: Uuid::new_v4(), imei: 351756051523999, lat: 0, lon: 0, alt: 0, angle: 0, satellites: 0, speed: 0.0, devicetime: 0}
    }
    
    pub async fn all(session: Arc<CassSession>) -> Self{
        GPSData{id: Uuid::new_v4(), imei: 351756051523999, lat: 0, lon: 0, alt: 0, angle: 0, satellites: 0, speed: 0.0, devicetime: 0}
    }
    
    pub async fn first_by_id(session: Arc<CassSession>) -> Self{
        GPSData{id: Uuid::new_v4(), imei: 351756051523999, lat: 0, lon: 0, alt: 0, angle: 0, satellites: 0, speed: 0.0, devicetime: 0}
    }

    pub async fn last(session: Arc<CassSession>) -> Self{
        GPSData{id: Uuid::new_v4(), imei: 351756051523999, lat: 0, lon: 0, alt: 0, angle: 0, satellites: 0, speed: 0.0, devicetime: 0}
    }
}
