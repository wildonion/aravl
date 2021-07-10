


use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use serde::Deserialize;
use serde_json::json;
use std::fmt;


#[derive(Debug, Deserialize)] //-- deserialize the structure
pub struct AVL{
    pub error_status_code: u16,
    pub error_message: String,
}


impl AVL{
    pub fn new(error_status_code: u16, error_message: String) -> AVL{ //-- constructor
        AVL{
            error_status_code,
            error_message,
        }
    }
}


impl fmt::Display for AVL{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        f.write_str(self.error_message.as_str())
    }
}

impl From<DieselError> for AVL{
    fn from(error: DieselError) -> AVL{
        match error{
            DieselError::DatabaseError(_, err) => AVL::new(409, err.message().to_string()), //-- this error will occure because of bad insert or update operation
            DieselError::NotFound => {
                AVL::new(404, "⚠️ not found".to_string())
            }
            err => AVL::new(500, format!("⚠️ unknown diesel error: {}", err)),
        }
    }
}

impl ResponseError for AVL{
    fn error_response(&self) -> HttpResponse{ //-- return a http response object
        // let status_code = StatusCode::from_u16(self.error_status_code).unwrap();
        let status_code = match StatusCode::from_u16(self.error_status_code){
            Ok(status_code) => status_code,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_message = match status_code.as_u16() < 500{
            true => self.error_message.clone(),
            false => "⚠️ internal server error".to_string(),
        };
        HttpResponse::build(status_code).json(json!({"message": error_message}))
    }
}