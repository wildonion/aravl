



use actix_web::{guard, get, post, web, HttpRequest, HttpResponse, Result};
use crate::handlers::error::AVL;
use serde_json::json;
use super::model::{InsertableUser, QueryableUser}; //-- load from the root of the current crate
use uuid::Uuid;
use std::sync::{Arc, Mutex};



#[get("/avl/api/auth/users")]
async fn find_all() -> Result<HttpResponse, AVL>{
    let users = QueryableUser::find_all().await?;
    Ok(HttpResponse::Ok().json(users))
}

#[get("/avl/api/auth/user/get/{id}")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, AVL>{
    let user = QueryableUser::find(id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}


#[post("/avl/api/auth/user/add")]
async fn add(user: web::Json<InsertableUser>) -> Result<HttpResponse, AVL>{
    let user = QueryableUser::add(user.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[post("/avl/api/auth/user/edit/{id}")]
async fn update(id: web::Path<Uuid>, user: web::Json<InsertableUser>) -> Result<HttpResponse, AVL>{
    let user = QueryableUser::update(id.into_inner(), user.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[post("/avl/api/auth/user/delete/{id}")]
async fn delete(id: web::Path<Uuid>) -> Result<HttpResponse, AVL>{
    let deleted_user = QueryableUser::delete(id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({"deleted": deleted_user})))
}

pub fn init_service(config: &mut web::ServiceConfig){
    config.service(find_all);
    config.service(find);
    config.service(add);
    config.service(update);
    config.service(delete);
}