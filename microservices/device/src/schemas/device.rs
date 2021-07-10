




use crate::authenticity;
use crate::utils;
use crate::handlers::db::cass::stablish::CassSession;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::result;
use std::str;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use cdrs_tokio::query::*;
use cdrs_tokio::query_values;
use cdrs_tokio::frame::AsBytes;
use cdrs_tokio::types::from_cdrs::FromCdrsByName;
use cdrs_tokio::types::prelude::*;
use cdrs_tokio_helpers_derive::*;
use reqwest::header;



pub mod Map{

    ///////////// =====================================================================================================================================================================
    /////////////                                       MAPPING THE HEX ADDRESS OF THE FILLED BUFFER WITH UTF-8 DIRECTLY INTO GPSMem STRUCTURE
    /////////////                                                   https://pramode.in/2016/09/13/using-unsafe-tricks-in-rust/
    /////////////
    ///////////// 
    /////////////
    ///////////// a str is conceptually just an immutable row of u8 bytes of dynamic length somewhere in memory (heap, stack or in the binary) 
    ///////////// the size the str takes up cannot be known at compile time and depends on runtime information it cannot be stored in 
    ///////////// a variable because the compiler needs to know at compile time what the size of each variable is with the guarantee 
    ///////////// that it forms valid UTF-8. how large is the row? no one knows until runtime hence it can't be stored in a variable 
    ///////////// it can only be handled it behind a pointer, this means that str most commonly appears as &str, so let name = "wildonion" is &str by default.
    ///////////// String is A growable, ownable heap-allocated data structure that owns the actual full str buffer on the heap it manages and it can be coerced to a &str.
    ///////////// &str substrings (a slice of String) are just fat pointers to that buffer (allocated by String) on the heap.
    ///////////// Prefer String if you want to own or mutate a string such as passing the string to another thread, etc.
    ///////////// Prefer &str if you want to have a read-only view of a string.
    ///////////// :) I think that it was str at first and because of its unknown size, String was created to store the str on the heap and &str is a reference to that heap!
    ///////////// :) there is no implementation of Copy trait for String and &str!
    /////////////
    /////////////
    ///////////// trait Sized is not implemented for [u8] or Vec<u8> from rust point-of-view, to solve this issue either we have to take a reference from [u8] or use 
    ///////////// C pointers like *const and *mut to map the GPS buffer into GPSMem structure inside the union, due to unsafeness manner of these raw pointers in rust 
    ///////////// we won't face the compile time size issue cause these raw pointers don't check anything! inorder to get the data of the GPS, all we have to do 
    ///////////// is just dereference the data field cause we have the C pointer address of the buffer inside the data field too 
    ///////////// and it's like our *const u8 is also our *mut GPSMem due to union structure!
    ///////////// the hex address of the buffer in memory on every incoming data through socket is different due to filling the stack with every push of new GPS data.
    ///////////// we could rather fill the union buffer with only one field like : buffer[0..buffer_bytes].as_ptr() as *const u8 as *mut GPSMem
    ///////////// but we casted the u8 bytes of the filled buffer using as_ptr() into a C pointer containing the hex address of all u8 bytes in memory which 
    ///////////// will return a *const, then by using a union we'll fill the GPSMem structure fields using the hex address of the filled buffer,
    ///////////// literally we mapped the memory of the GPS structure in device side into pre-defined structure in rust side!
    ///////////// we want to map the hex address of our filled buffer into our GPSMem structure in memory, to do that in a nitty gritty manner in rust, we must get the hex address 
    ///////////// of the filled buffer by calling the as_ptr() function on the filled buffer, which will return a C based raw immutable pointer; *const u8.
    ///////////// also we want to save our GPS data along with hex address of our buffer into a same location in memory, thus a union might be a great data structure
    ///////////// to map the hex address of the buffer into the GPSMem struct at the same time and in a same location! 
    ///////////// based on what we did to our buffer we must defined the type for each field as *mut and *const respectively, cause unions share the same memory location for all its 
    ///////////// fields means u8 can also be u16 or even String or &str, in our case *const can also be *mut; and the size of each union depends on the largest size field,
    ///////////// the size of our GPS union is equal to the size of its largest field which is GPSMem structure cause our GPS device might not send some fields.
    ///////////// &T reference will implicitly coerce to an *const T raw pointer in safe code and similarly for the mut variants : &mut T is coerced to *mut T
    ///////////// (both coercions can be performed explicitly with, respectively, value as *const T and value as *mut T).
    ///////////// transmute function tells rust to treat a value of one type as though it were another type.
    /////////////
    /////////////  
    ///////////// =====================================================================================================================================================================
    
    
    
    
    /////////////////////////////////////////////////// GPS UNION TO MAP THE HEX ADDRESS OF THE BUFFER INTO GPSMem STRUCTURE //////////////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
    #[repr(C)] //-- this is the most important repr - do what C does, the order, size, and alignment of fields is exactly what you would expect from C or C++
    union GPS{
        ///////////// BUG-ALERT /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        ///////////// if we don't put the *mut before GPSMem it'll map the C pointer address of buffer field into the data field on every incoming bytes from the GPS socket.
        ///////////// can't share references of C pointers between threads using Arc or have multiple owner with Rc, cause they are not safe!
        ///////////// std::ptr::null() returns a null pointer in memory
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        pub data: *mut self::GPSMem, //-- a C based raw mutable pointer to GPSMem
        pub buffer: *const u8, //-- a C based raw immutable pointer to u8, cause tokio fill the buffer in u8 format by default and also the default type of as_ptr() is *const T where T is the typr that we're calling the as_ptr() method on
    }
    
    
    
    
    /////////////////////////////////////////////////// GPS MEMORY MAPPING MODEL /////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////// https://wiki.teltonika-gps.com/view/Teltonika_Data_Sending_Protocols ////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
    #[derive(Copy, Clone, Debug)] //-- every Copy type is also required to be Clone thus we'll have the GPSData struct even after we moved it into another variable cause the trait Copy is implemented for that
    #[repr(C)] // UNSAFE-ALERT - #[repr(C, packed)] : borrow of packed field is unsafe and requires unsafe function or block
    pub struct GPSMem{ //-- GPSMem will be stored on the stack due to its static data types thus it's bounded to the Copy trait 
        pub imei: i16,
        pub lat: i32,
        pub lon: i32,
        pub alt: i16,
        pub angle: i16,
        pub satellites: i8,
        pub speed: i16,
        pub timestamp: u64,
    }


    pub fn hex_to_gps_mem_struct(buffer: &[u8]) -> Option<&mut GPSMem>{
        unsafe impl Send for GPS {} //-- due to unsafeness manner of C based raw pointers we implement the Send trait for GPS union in order to be shareable between tokio threads
        unsafe{
            let mut gps_data: Option<&mut GPSMem> = None; //-- in order to prevent from null checking we took a safe rusty based reference from GPSMem inside the Option (cause rust doesn't have null) cause *mut raw pointer to GPSMem inside union is not safe also can't move between thread safely and once the GPS data has dereferenced it might return a null pointer due to unsafe manner of C pointers
            let gps = GPS{buffer: buffer.as_ptr() as *const u8}; //-- getting a C pointer of the filled buffer which is the hex address of the memory, doning this is unsafe due to unsafeness manner of raw pointers in rust
            println!("\t>>>>> GPS buffer as u8 (UTF-8)                : {:?}", &buffer);
            println!("\t>>>>> GPS buffer hex address in server memory : {:p}", &buffer); // NOTE - this address is same as the value of buffer field inside the union cause accessing rust is an unsafe operation thus we can only print it
            println!("\t>>>>> GPS buffer hex address in union         : {:p}", gps.buffer); // NOTE - gps.buffer contains the pointer to the heap data location of &[u8] which is a raw pointer as *const u8
            gps_data = Some(&mut (*gps.data)); //-- taking a reference (smart pointer to address of gps.data in the stack) from dereferenced *mut raw pointer from the union inside the unsafe block is done through * like in C syntax  - we only wants the data so we didn't do any read operation on buffer field inside the union
            gps_data
        } 
    }

}



/////////////////////////////////////////////////// GPS CONFIGURATION MODEL //////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
#[derive(Debug, Clone, Serialize, Deserialize)] //-- it can only be cloned cause trait Copy is not implemented thus we can't have a copy of GPSConfig struct and when we assign it to new variable it'll move
pub struct GPSConfig{
    pub ip: Option<String>,
    pub port: Option<String>,
    pub tts: Option<String>,
}


/////////////////////////////////////////////////// CASSANDRA GPS DATA MODEL /////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////// https://github.com/krojew/cdrs-tokio/blob/master/type-mapping.md ////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
#[derive(Serialize, Deserialize, Copy, Clone, Debug, IntoCdrsValue, TryFromRow, TryFromUdt)] //-- every Copy type is also required to be Clone thus we'll have the GPSData struct even after we moved it into another variable cause the trait Copy is implemented for that
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


    ///////////// =======================================
    ///////////// convert UTF-8 bytes to hex ascii string 
    ///////////// =======================================

    pub fn u82hex(buffer: &[u8]) -> result::Result<String, ()>{
        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //////////// DATA FORMAT : 000000000000003608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF
        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // let decoded_hex_ascii_string_to_u16 = utils::from_hex_string_to_u16(utils::from_u8_to_hex_string(&buffer[0..buffer_bytes]).as_str()).expect("⚠️ hex to u16 error");
        // let parsed_hex_ascii_string_to_u8 = utils::from_hex_string_to_u8(utils::from_u8_to_hex_string(&buffer[0..buffer_bytes]).as_str()); //-- coerce String to str or &str - returns the hex ascii string in a vector of u8 format
        // let decoded_hex_ascii_string_to_u8 = hex::decode(utils::from_u8_to_hex_string(&buffer[0..buffer_bytes])).expect("⚠️ u8 to hex error");
        // let hex_ascii_string_using_hex = hex::encode(&buffer);
        // hex::decode(&hex_ascii_string_using_hex)
        utils::from_u8_to_hex_string(&buffer)
    }


    ///////////// ===========================
    ///////////// convert UTF-8 bytes to &str 
    ///////////// ===========================

    pub fn u82str(buffer: &[u8]) -> result::Result<&str, std::str::Utf8Error>{
        match str::from_utf8(&buffer){
            Ok(data) => { Ok(data) },
            Err(e) => { Err(e) }
        }
    }


    ///////////// ===========================================
    ///////////// parsing &str to extract the data parameters
    ///////////// ===========================================

    pub fn parse_from_str(data: &str, device_addr: SocketAddr) -> result::Result<Vec<HashMap<&str, &str>>, ()>{
        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //////////// DATA FORMAT : imei=000000000000&lat=0000000.0000&lon=000000.00000&alt=00000.0000&timestamp=0000-00-00T00:00:00.0000
        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        let mut hash_map_data_vector: Vec<HashMap<&str, &str>> = vec![HashMap::new()];
        if data.contains("HTTP") || !data.contains("&"){
            Ok(hash_map_data_vector)
        } else{
            let splitted_data_by_crlf: Vec<&str> = data.split("\r\n").collect(); //-- splitting incoming data by \r\n cause it might be sent multiple data at a time from the GPS, maybe the data was stuck!
  
            /* --------------- GETTING THE LAST GPS DATA TO ADD TIMESTAMP ---------------
                let data = { //-- get the last data sent through the GPS socket
                    if splitted_data_by_crlf.last().expect("⚠️ can't get the last splitted GPS data").is_empty(){ //-- if one data is sent definitely the first element is the only one that we need
                        *splitted_data_by_crlf.first().expect("⚠️ can't get the last splitted GPS data") //-- dereferencing the last element to get &str cause last() will return a reference to the reference to the Vec<&str> which is &&str
                    } else{ //-- if multiple data is sent definitely the last one is the one that we need 
                        *splitted_data_by_crlf.last().expect("⚠️ can't get the last splitted GPS data") //-- dereferencing the last element to get &str cause last() will return a reference to the reference to the Vec<&str> which is &&str
                    }
                };
                let received_data_server_time = chrono::Local::now().naive_local();
                let received_data_server_time_param = format!("&timestamp={:?}", received_data_server_time);
                let mut string_data_trimmed = data.to_string();
                let len_withoutcrlf = string_data_trimmed.to_string().trim_end().len(); //-- getting the length of the trimmed data - removing crlf or \r\n from the end
                string_data_trimmed.truncate(len_withoutcrlf); //-- truncating to the legnth of the trimmed data
                let data = format!("{}{}", string_data_trimmed, received_data_server_time_param);
            ---------------------------------------------------------------------------- */
            
            for data in splitted_data_by_crlf{ //-- we need all incoming data from the GPS so we must iterate through each of them to extract the params inside every data
                if data.is_empty(){ continue; } //-- if this occurs means we have only one incoming data from the GPS cause the second element splitted by \r\n is an empty string 
                else{ //-- if we fall down in here means that we got a data from the GPS so we have to parse it
                    println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | GPS DATA : {:?}", device_addr, chrono::Local::now().naive_local(), data); //-- logging the data on server
                    match utils::param_parser(&data, "&", "="){ //-- data is String and will be coerce to &str when it's passing to function
                        Ok(parsed_into_hash_map) => { hash_map_data_vector.push(parsed_into_hash_map); },
                        Err(_) => {} //-- param_parser() utility function returns () on Result error type thus we can't return any error value inside Err arm in here  
                    }
                }
            }
        Ok(hash_map_data_vector)
        }
    }


    ///////////// ===========================================================
    ///////////// checking device authenticity for sending GPS configurations 
    ///////////// ===========================================================

    pub async fn check_device(data_body: &'static str) -> result::Result<GPSConfig, reqwest::Error>{
        let token = "";
        let mut headers = header::HeaderMap::new();
        // TODO - add necessary headers if needed
        // ... 
        // headers.insert("AUTHORIZATION", header::HeaderValue::from_static(token));
        // headers.insert("USER_AGENT", header::HeaderValue::from_static("AGP101/v1.00!"));
        // headers.insert("ACCEPT_ENCODING", header::HeaderValue::from_static("gzip, deflate"));
        // headers.insert("ACCEPT", header::HeaderValue::from_static("*/*"));
        // headers.insert("CONNECTION", header::HeaderValue::from_static("keep-alive"));
        // headers.insert("CONTENT_LENGTH", header::HeaderValue::from_static("2"));
        // headers.insert("CONTENT_TYPE", header::HeaderValue::from_static("application/json"));
        match authenticity!(data_body, headers){ //-- calling api of auth microservice to check the device authenticity
            Ok(gps_config) => { Ok(gps_config) },
            Err(e) => { Err(e) }
        }
    }


    ///////////// =========================================
    ///////////// sending GPS configurations back to device 
    ///////////// =========================================

    pub async fn send_config(config_data: GPSConfig) -> String{
        if let ( Some(ip), Some(port), Some(tts) ) = (config_data.ip, config_data.port, config_data.tts){ //-- crashing prevention solution on None ip, port and tts
            // TODO - send the number of total bytes of GPSConfig struct or the hash of the GPS configurations along with the GPS configurations data
            // ...
            format!("{}\"status\": 200,{},{},{} {}" ,'{' ,ip, port, tts,'}')
        } else{
            format!("{}\"status\": 204 {}", '{', '}') //-- default value of GPS configurations are set to a specified value in postgres db
        }
    }


    ///////////// ===============================
    ///////////// GPSData cassandra model methods 
    ///////////// ===============================

    fn insert(&self) -> QueryValues{
        query_values!("id" => self.id, "imei" => self.imei, "lat" => self.lat, "lon" => self.lon, "alt" => self.alt, "angle" => self.angle, "satellites" => self.satellites, "speed" => self.speed, "devicetime" => self.devicetime)
    }



    pub async fn init(session: Arc<CassSession>){
        let create_gps_data_table = "CREATE TABLE IF NOT EXISTS avl.gps_data (id UUID, imei bigint, lat float, lon float, alt float, angle float, satellites int, speed float, devicetime timestamp, servertime timestamp, PRIMARY KEY((id, imei), devicetime, servertime));";
        session.query(create_gps_data_table).await.expect("⚠️ gps_data table creation error");
    }



    pub async fn save(&self, session: Arc<CassSession>){
        let insert_gps_data_cql = "INSERT INTO avl.gps_data (id, imei, lat, lon, alt, angle, satellites, speed, devicetime, servertime) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, toUnixTimestamp(now()))";
        let values = self.insert();
        session.query_with_values(insert_gps_data_cql, values).await.expect("⚠️ gps_data column family insertion error");
    }
}

