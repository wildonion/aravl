







use diesel::prelude::*;
use diesel::{Insertable, Queryable, AsChangeset};
use crate::handlers::error::AVL;
use crate::schema::users;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::handlers::db::pg::stablish as pg;





///////////// =========================================================================================================================
#[derive(Deserialize, Insertable, AsChangeset)] //-- we've used deserialize trait to deserialize user request from json to InsertableUser struct
#[table_name="users"]
pub struct InsertableUser{ //-- we'll use this struct to deserialize and insert into postgres
    pub id: Option<Uuid>,
    pub national_code: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub mac: String,
    pub sex: Option<String>,
    pub age: Option<i16>,
    pub reg_date: Option<chrono::NaiveDateTime>,
}

impl InsertableUser{
    fn from(user: InsertableUser) -> InsertableUser{
        InsertableUser{
            id: Some(Uuid::new_v4()),
            national_code: user.national_code,
            first_name: user.first_name,
            last_name: user.last_name,
            phone_number: user.phone_number,
            mac: user.mac,
            sex: user.sex,
            age: user.age,
            reg_date: Some(chrono::Local::now().naive_local()),
        }
    }
}
///////////// ==========================================================================================================================
#[derive(Serialize, Queryable, AsChangeset)] //-- we've used serialize trait to serialize new input from QueryableUser struct to json to send it through actix
#[table_name="users"]
pub struct QueryableUser{ //-- we'll use this struct to fetch from postgres then serialize
    pub id: Uuid,
    pub national_code: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub mac: String,
    pub sex: String,
    pub age: i16,
    pub reg_date: chrono::NaiveDateTime,
}

impl QueryableUser{
    
    pub async fn find_all() -> Result<Vec<Self>, AVL>{
        let conn = pg::connection().await?;
        let users = users::table.load::<QueryableUser>(&conn)?;
        Ok(users)
    }

    pub async fn find(id: Uuid) -> Result<Self, AVL>{
        let conn = pg::connection().await?;
        let user = users::table.filter(users::id.eq(id)).first::<QueryableUser>(&conn)?;
        Ok(user)
    }

    pub async fn add(user: InsertableUser) -> Result<Self, AVL>{
        let conn = pg::connection().await?;
        let user = InsertableUser::from(user);
        let user = diesel::insert_into(users::table).values(user).get_result(&conn)?;
        Ok(user)
    }

    pub async fn update(id: Uuid, user: InsertableUser) -> Result<Self, AVL>{ //-- Self refers to the User type
        let conn = pg::connection().await?;
        let user = diesel::update(users::table.filter(users::id.eq(id))).set(user).get_result(&conn)?;
        Ok(user)
    }

    pub async fn delete(id: Uuid) -> Result<usize, AVL>{
        let conn = pg::connection().await?;
        let response = diesel::delete(users::table.filter(users::id.eq(id))).execute(&conn)?; //-- usize is the size of allocated bytes in memory to take a reference from any type like on i32 is 4 bytes
        Ok(response)
    }
}
///////////// =========================================================================================================================

