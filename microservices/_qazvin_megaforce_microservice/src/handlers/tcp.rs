
use std::collections::HashMap;
use std::str;
use std::sync::Arc;
use std::net::SocketAddr;
use crate::utils;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde_json::{Result, Value};
use reqwest::header;


pub async fn process_connection(mut socket: TcpStream, mut buffer: Vec<u8>, device_addr: SocketAddr){


    loop { //-- keep socket always open to receive all new incoming data from every connected GPS and won't repeat until all async functions in here completes, if there was no loop the socket would be closed after receiving one data from the GPS - if you want to receive all 20 GB of data at once through socket you must have at least 20 GB of RAM!


            match socket.read(&mut buffer).await{ //-- read incoming bytes from the TcpStream socket and put them in the buffer



                ///////////// =======================================================================
                ///////////// device is disconnected from our server thus no incoming bytes are exist 
                ///////////// =======================================================================
                
                        Ok(buffer_bytes) if buffer_bytes == 0 => { //-- if the buffer_bytes is 0 means we're about to close the socket 
                            println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ❌", device_addr, chrono::Local::now().naive_local());
                            return; //-- close the socket for the device
                        },
                        
                        
                        
                        ///////////// ========================================
                        ///////////// filling buffer with incoming UTF-8 bytes
                        ///////////// ========================================
                        
                        Ok(buffer_bytes) => { // NOTE - tokio converts incoming bytes in u8 or u16 or hex ascii from every GPS into big endian UTF-8 bytes

                           



                            ///////////// ==============================
                            ///////////// converting UTF-8 bytes to &str
                            ///////////// ==============================

                                match str::from_utf8(&buffer[0..buffer_bytes]){
                                    Ok(data) => {
                                        let splitted_data_by_crlf: Vec<&str> = data.split("\r\n").collect(); //-- splitting incoming data by \r\n cause it might be sent multiple data at a same time from the GPS, maybe the data was stuck!
                                        let data = { //-- get the last data sent through the GPS socket
                                            if splitted_data_by_crlf.last().expect("⚠️ can't get the last splitted GPS data").is_empty(){ //-- if one data is sent definitely the first element is the only one that we need
                                                *splitted_data_by_crlf.first().expect("⚠️ can't get the last splitted GPS data")
                                            } else{ //-- if multiple data is sent definitely the last one is the only one that we need 
                                                *splitted_data_by_crlf.last().expect("⚠️ can't get the last splitted GPS data")
                                            }
                                        };
                                        let received_data_server_time = chrono::Local::now().naive_local();
                                        let received_data_server_time_url_param = format!("&timestamp={:?}", received_data_server_time);
                                        let mut string_data_trimmed = data.to_string();
                                        let len_withoutcrlf = string_data_trimmed.to_string().trim_end().len(); //-- getting the length of the trimmed data - removing crlf or \r\n from the end
                                        string_data_trimmed.truncate(len_withoutcrlf); //-- truncating to the legnth of the trimmed data
                                        let data = format!("{}{}", string_data_trimmed, received_data_server_time_url_param);
                                        println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | GPS DATA : {:?}", device_addr, chrono::Local::now().naive_local(), data);
                                        let params = utils::param_parser(&data, "&", "=").expect("⚠️ can't parse url params &str"); //-- data is String and will be coerce to &str





                                        ///////////// ====================== POST GPS DATA TO INOBI TRACCAR SERVER IF THE NUMBER OF SATTELITES ARE EQUALS OR MORE THAN 4 ======================
                                        // TODO - check NS param value before sending it to traccar
					// ...
					// match reqwest::Client::builder().build(){
                                        //     Ok(client) => {
                                        //         let url = format!("http://localhost:5055/?{}", data); 
                                        //         match client.post(url).send().await{ // NOTE - client future won't solve unless we await on it and once it gets solved we'll fall into the following arms
                                        //             Ok(res) => {
                                        //                 match res.text().await{
                                        //                     Ok(json) => {
                                        //                         println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | RESPONSE DATA FROM TRACCAR SERVER : {:?}", device_addr, chrono::Local::now().naive_local(), json);          
                                        //                     },
                                        //                     Err(e) => {
                                        //                         println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | RESPONSE DATA FROM TRACCAR SERVER : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                        //                     }
                                        //                 }
                                        //             },
                                        //             Err(e) => {
                                        //                 println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | TRACCAR SERVER STATUS : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                        //             }
                                        //         }
                                        //     },
                                        //     Err(e) => {
                                        //         println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | HTTP CLIENT OBJECT STATUS : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                        //     }
                                        // }
                                        ///////////// ====================== POST GPS DATA TO INOBI PYTHON SERVER ======================
                                        
                        				match reqwest::Client::builder().build(){
                                                           Ok(client) => {
                                                               let url = format!("http://78.38.56.34:8282/transport/bus?{}", data);
                                                               match client
                                                                   .post(url)
                                                                   .header(header::AUTHORIZATION, "Bearer ZXlKMGVYQWlPaUpLVjFRaUxDSmhiR2NpT2lKSVV6STFOaUo5LmV5SjBhVzFsSWpveE5UTXdNVEF4T0RNM0xDSnpZMjl3WlhNaU9sc2lkSEpoYm5Od2IzSjBYM1Z1YVhRaUxDSjBjbUZ1YzNCdmNuUmZkVzVwZENoa1pXWmhkV3gwS1NKZExDSnBZWFFpT2pFMU16QXdPVGs1TWpRc0ltbHpjeUk2SW1seWMyRnNZV0prUUdkdFlXbHNMbU52YlNKOS45X1RlUGw3SGowckhEemhlMWpfal9mWkRCNWN2SjRyOXJ6R2ZDM1FhT0lJ")
                                                                   .header(header::USER_AGENT, "AGP101/v1.00!")
                                                                   .header(header::ACCEPT_ENCODING, "gzip, deflate")
                                                                   .header(header::ACCEPT, "*/*")
                                                                   .header(header::CONNECTION, "keep-alive")
                                                                   .header(header::CONTENT_LENGTH, "2")
                                                                   .header(header::CONTENT_TYPE, "application/json ")
                                                                   .body("{}")
                                                                   .send()
                                                                   .await{ // NOTE - client future won't solve unless we await on it and once it gets solved we'll fall into the following arms
                                                                       Ok(res) => {
                                                                           match res.text().await{ //-- do other stuff with the text response
                                                                               Ok(json) => {
                                                                                   // --------------------------------------------------------------------------------------------------------
                                                                                   println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | RESPONSE DATA FROM INOBI SERVER : {}", device_addr, chrono::Local::now().naive_local(), json);
                                                                                   let mut ip: Option<&str> = None;
                                                                                   let mut port: Option<&str> = None;
                                                                                   let mut tts: Option<&str> = None;
                                                                                   let params: Vec<&str> = json.split(",").collect();
                                                                                   for p in params{
                                                                                       if p.contains("\"port\":"){ port = Some(p); } //-- check that port is in the splitted parameters or not
                                                                                       else if p.contains("\"ip\":"){ ip = Some(p); } //-- check that ip is in the splitted parameters or not
                                                                                       else if p.contains("\"tts\":"){ tts = Some(p); } //-- check that tts is in the splitted parameters or not
                                                                                   }
                                                                                   // --------------------------------------------------------------------------------------------------------
                                                                                   let response = if let ( Some(ip), Some(port), Some(tts) ) = (ip, port, tts){ //-- crashing prevention solution on None ip, port and tts
                                                                                       println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | IP, PORT & TTS EXTRACTED FROM RESPONSE : {} {} {}", device_addr, chrono::Local::now().naive_local(), ip, port, tts);
                                                                                       let mut response = format!("{}\"status\": 200,{},{},{},END{}" ,'{', ip, port, tts, '}');
                                                                                       let count = response.as_bytes().len();
                                                                                       response = format!("\"count\": {}{}", count, response);
                                                                                       response
                                                                                   } else {
                                                                                       println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | IP, PORT & TTS NOT EXIST IN RESPONSE", device_addr, chrono::Local::now().naive_local());
                                                                                       format!("\n\"status\": 200")
                                                                                   };
               
                                                                                   // --------------------------------------------------------------------------------------------------------
                                                                                   match socket.write_all(response.as_str().as_bytes()).await{ //-- writing the response into the socket buffer to send back to the GPS - this method will continuously call write until there is no more data to be written
                                                                                       Ok(buffer_bytes) => {
                                                                                           println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CONFIG DATA SENT BACK TO SOCKET WITH STATUS : {}", device_addr, chrono::Local::now().naive_local(), 200);
                                                                                       },
                                                                                       Err(e) => {
                                                                                           println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | CONFIG DATA SENT BACK TO SOCKET WITH STATUS : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                                                                       }
                                                                                   }
                                                                                   // --------------------------------------------------------------------------------------------------------
                                                                               },
                                                                               Err(e) => {
                                                                                   println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | RESPONSE DATA FROM INOBI SERVER : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                                                               }
                                                                           }
                                                                       },
                                                                       Err(e) => {
                                                                           println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | INOBI SERVER STATUS : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                                                       }
                                                                   }
                                                           },
                                                           Err(e) => {
                                                               println!("[+] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | HTTP CLIENT OBJECT STATUS : {:?}", device_addr, chrono::Local::now().naive_local(), e);
                                                           }
                                                       }
                                                       ///////////// ==================================================================================================================
                                                   },
                                                   Err(e) => {
                                                       println!("[!] GPS : {} | CURRENT SERVER TIME : {:?} | STATUS : ✅ | DATA : {:?}", device_addr, chrono::Local::now().naive_local(), e);
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
