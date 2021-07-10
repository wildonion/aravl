

use std::collections::HashMap;
use std::str;
use std::sync::Arc;
use std::net::SocketAddr;
use crate::utils;
use crate::handlers::db::cass::stablish::CassSession;
use crate::schemas::device::{GPSData, GPSConfig, Map};
use crate::handlers::db::pg::stablish::Pool;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;



pub async fn process_connection(mut socket: TcpStream, cass_session: Arc<CassSession>, pg_pool: Pool, mut buffer: Vec<u8>, device_addr: SocketAddr){


    loop { //-- keep socket always open to receive all new incoming data from every connected GPS and won't repeat until all async functions in here completes, if there was no loop the socket would be closed after receiving one data from the GPS - if you want to receive all 20 GB of data at once through socket you must have at least 20 GB of RAM!


            match socket.read(&mut buffer).await{ //-- read incoming bytes from the TcpStream socket and put them in the buffer



                ///////////// =======================================================================
                ///////////// device is disconnected from our server thus no incoming bytes are exist
                /////////////           NOTE - WE MUST CHECK THIS FOR TCP CONNECTIONS
                ///////////// =======================================================================
                
                        Ok(buffer_bytes) if buffer_bytes == 0 => { //-- if the buffer_bytes is 0 means we're about to close the socket 
                            println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ❌", device_addr, chrono::Local::now().naive_local());
                            return; //-- close the socket for the device
                        },
                        
                        
                        
                        ///////////// ========================================
                        ///////////// filling buffer with incoming UTF-8 bytes
                        ///////////// ========================================
                        
                        Ok(buffer_bytes) => { // NOTE - tokio converts incoming bytes in u8 or u16 or hex ascii from every GPS into big endian UTF-8 bytes

                            
                            ///////////// ==================================================================
                            ///////////// mapping the hex address of UTF-8 bytes into union GPSMem structure 
                            ///////////// ==================================================================
                                /* --------------------------------------------------------------------------------------------------------------------------
                                match Map::hex_to_gps_mem_struct(&buffer[0..buffer_bytes]){
                                    Some(gps_data) => {
                                        // TODO - if we got the GPS configurations data then we know that the device is registered in postgres database and we must save it into cassandra 
                                        // TODO - build an instance from GPSData struct by UART mapping
                                        // ...
                                        let received_data_server_time = chrono::Local::now().naive_local();
                                        let id = Uuid::new_v4();
                                        let imei = gps_data.imei as i64;
                                        let lat = gps_data.lat;
                                        let lon = gps_data.lon;
                                        let alt = gps_data.alt;
                                        let angle = gps_data.angle;
                                        let satellites = gps_data.satellites;
                                        let speed = gps_data.speed as f32;
                                        let devicetime = gps_data.timestamp as i64;
                                        let gps_data_struct = GPSData{id, imei, lat, lon, speed, alt, angle, satellites, devicetime};
                                        println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | MAPPED DATA : {:?}", device_addr, chrono::Local::now().naive_local(), gps_data_struct);
                                        gps_data_struct.save(cass_session.clone()).await; //-- gps_data_struct.save() future will never be solved unless we await on it and will never block the current thread on its executation once we are awaited
                                    },
                                    None => {
                                        println!("⚠️ Can't map the hex address of the buffer into union field thus couldn't fill the GPSMem struct");
                                    }
                                }
                                -------------------------------------------------------------------------------------------------------------------------- */



                            ///////////// ====================================================================
                            ///////////// converting UTF-8 bytes to hex ascii string based on teltonika format
                            ///////////// ====================================================================
                            
                                match GPSData::u82hex(&buffer[0..buffer_bytes]){
                                    Ok(hex_ascii_string) => {
                                        // TODO - if we got the GPS configurations data then we know that the device is registered in postgres database and we must save it into cassandra 
                                        // TODO - build an instance from GPSData struct by parsing hex ascii string
                                        // TODO - parse the following hexadecimal stream based on https://wiki.teltonika-gps.com/view/Teltonika_Data_Sending_Protocols format
                                        // 000000000000003608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF
                                        // ...
                                        // let received_data_server_time = chrono::Local::now().naive_local();
                                        // let id = Uuid::new_v4();
                                        // let imei = i64::from_str_radix(&hex_ascii_string[9..16], 16); //-- converting a string slice in a 16 base to i32 integer
                                        // let gps_data_struct = GPSData{id, imei, lat, lon, speed, alt, angle, satellites, devicetime};
                                        // println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | GPS DATA : {:?}", device_addr, chrono::Local::now().naive_local(), gps_data_struct);
                                        // gps_data_struct.save(cass_session.clone()).await; //-- gps_data_struct.save() future will never be solved unless we await on it and will never block the current thread on its executation once we are awaited
                                    },
                                    Err(e) => { //-- we didn't return an error type inside u82hex() method in its Result 
                                        println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CAN'T ENCODE FROM UTF-8 TO HEX : {:?}", device_addr, chrono::Local::now().naive_local(), e);   
                                    }
                                }



                            ///////////// ==============================
                            ///////////// converting UTF-8 bytes to &str
                            ///////////// ==============================

                                match GPSData::u82str(&buffer[0..buffer_bytes]){

                                    Ok(data) => {
                                        
                                    ///////////// ================================
                                    ///////////// parsing incoming params from GPS
                                    ///////////// ================================

                                        match GPSData::parse_from_str(&data, device_addr){

                                            Ok(data_params) => {
                                                

                                                
                                                if data_params[0].is_empty(){
                                                    println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ❌ | FOUND INVALID CONTENT IN SOCKET", device_addr, chrono::Local::now().naive_local());
                                                    break; //-- close the socket cause it might be a hacker!!!!
                                                }




                                                
                                                

                                                let data_body: &str = ""; // TODO - contains imei and secret key
                                                match GPSData::check_device(data_body).await{
                                                    
                                                    Ok(gps_config) => {
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        
                                                        // TODO - if we got the GPS configurations data then we know that the device is registered in postgres database and we must save it into cassandra 
                                                        // TODO - build an instance from GPSData struct using data_params
                                                        // ...
                                                        // gps_data_struct.save(cass_session.clone()).await; //-- gps_data_struct.save() future will never be solved unless we await on it and will never block the current thread on its executation once we are awaited
                                                        
                                                        





                                                        
                                                        let config_response = GPSData::send_config(gps_config).await; //-- awaiting on future will start to solve it in a none blocking manner
                                                        println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | GPS CONFIGURATIONS ARE SET TO : {}", device_addr, chrono::Local::now().naive_local(), config_response);
                                                        match socket.write_all(config_response.as_str().as_bytes()).await{ //-- writing the response into the socket buffer to send back to the GPS - this method will continuously call write until there is no more data to be written
                                                            Ok(buffer_bytes) => {
                                                                println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CONFIG DATA SENT BACK TO SOCKET WITH STATUS : {}", device_addr, chrono::Local::now().naive_local(), 200);
                                                            },
                                                            Err(e) => {
                                                                println!("[+] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CAN'T SEND DATA BACK TO SOCKET : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                                            }
                                                        }
                                                     









                                                    },
                                                    Err(e) => { //-- we returned a reqwest::Error type inside check_device() method in its Result
                                                        println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | DEVICE NOT REGISTERED : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                                    }
                                                }
                                            },
                                            Err(e) => { //-- we didn't return an error type inside parse_from_str() method in its Result
                                                println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CAN'T PARSE GPS DATA : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                            }
                                        }
                                    },
                                    Err(e) => { //-- we returned an Utf8Error type inside u82str() method in its Result 
                                        println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CAN'T DECODE FROM UTF-8 TO str : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                    }

                                }

                        },

                        

                ///////////// ======================
                ///////////// can't read from socket
                ///////////// ======================
                
                        Err(e) => {
                            println!("[!] CURRENT SERVER TIME : {:?} | STATUS : ❓ - {:?}", chrono::Local::now().naive_local(), e);
                            return; //-- close the socket for the device
                        }
            
            };

        
        }

}