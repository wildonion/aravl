







#[macro_use]
extern crate diesel;
mod entities;
mod handlers;
mod schema;
use actix_web::{App, web};
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use listenfd::ListenFd;
use actix_web::middleware::Logger;
use std::env;
use dotenv::dotenv;
use self::entities::users::api::init_service; //-- use crate::entities::users::api::init_service; | instead of passing init_service to configure function in actix HttpServer you can pass entities::users::api::init_service






async fn index() -> impl Responder {
    HttpResponse::Ok().body("<h1>Welcome 2 AVL Auth API</h1>")
}





#[actix_web::main]
async fn main() -> std::io::Result<()> { //-- return type is an empty Result object - std::io::Result is broadly used across std::io for any operation which may produce an error.
    




        /*
            all methods implemented in here are returned a Result object either Ok or AVL: Result<T, E>
            db::pg::stablish::connection().await? returns a connection selected from the pool of postgres connections
            futures won't solve unless we await on them.
            ? returns from the function with the Err, meaning that the function that includes ? syntax must return Result<T, E>
            unwrap panics on Err, meaning that the function can have any signature either<T, E> Result or Option<T>.
            expect panics if the value is an Err, with a panic message including the passed message, and the content of the Err.
        */
        env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
        env_logger::init();
        dotenv().expect("⚠️ .env file not found");
        let secret_key = env::var("JWT_SECRET_KEY").expect("⚠️ found no jwt secret key"); // TODO - 
        let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set"); // TODO - 









        // TODO - https://github.com/ryanmcgrath/jelly/
        // TODO - create organizations, roles and users table
        // TODO - this microservice is only for users and device authentication process
        // TODO - add /avl/api/auth/device route for device authenticity process
        // ...










        let mut listenfd = ListenFd::from_env();
        let mut server = 
            HttpServer::new(|| {
                App::new()
                    .configure(init_service)
                    .route("/avl/api/auth", web::get().to(index)) //-- default route
                    .wrap(Logger::default())
            });
        
        

        
        server = match listenfd.take_tcp_listener(0)?{
            Some(listener) => server.listen(listener)?,
            None => {
                let host = env::var("HOST").expect("⚠️ please set host in .env");
                let port = env::var("AUTH_PORT").expect("⚠️ please set port in .env");
                server.bind(format!("{}:{}", host, port))?
            }
        };

        server.run().await

}