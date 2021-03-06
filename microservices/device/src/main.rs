
///////////// ============================================================================================================
/////////////                   SPAWNING TOKIO MULTI THREADS PER DATA (TASK) COMMING FROM GPS SOCKET
/////////////                              https://tokio.rs/tokio/tutorial/shared-state
/////////////                                https://tokio.rs/tokio/tutorial/channels
/////////////
/////////////
///////////// there are multiple threads spawned for each GPS socket to run async tasks coming from each one. 
///////////// those tasks (multiple producers) will be sent through the job/task queue channel of tokio runtime 
///////////// to each threads (single consumer) and once a free thread sees the task it'll get the job and 
///////////// start executing it, if the task contains mutex variable the thread will lock on that variable 
///////////// and it won't unlock it until it's done with mutating it, otherwise deadlock will happen for 
///////////// threads! unlocking as the locking process is happening inside another thread.
///////////// loop {} inside the task closure means the GPS socket is listening constantly on every incoming 
///////////// data from the GPS to execute the job asynchronously inside a free thread, if this loop wasn't 
///////////// exist once we got a data from the GPS on first request the connection would be closed.
///////////// loop {} inside each thread means that thread is always listening to receive the new task from 
///////////// the job/task queue channel to execute it asynchronously, every time that we get a new data/job/task 
///////////// from the connected GPS through its socket we have to allocate new size for the buffer cause 
///////////// it must be filled with new data on every job.
/////////////
/////////////
///////////// jobs or tasks are Send when all the data that is held across .await calls are Send 
///////////// means the trait Send must be implemented for each type of that data we want to
///////////// share between threads, for example Send is not implemented for C based raw pointers like *const and *mut.
/////////////
/////////////
///////////// do not await on each tokio::spawn() cause it'll block the current thread, 
///////////// stuck inside the loop and doesn't allow other socket request come into the app 
///////////// until the the tokio future has run to completion, means it can't handle 
///////////// another incoming request at the beginning of the loop until the old one has resolved.
/////////////
/////////////
///////////// async prevent blocking the thread and execute codes in a none-blocking manner 
///////////// inside the thread, by doing this we allow other tasks inside the channel queue 
///////////// to take over the current thread if some task is blocked and run inside another thread.
/////////////
/////////////
///////////// GPS data is a task or a job or future which are handled constantly coming from 
///////////// every GPS socket and execute concurrently between threads and asynchronously to 
///////////// done it's job by spawning multi threads for each task.
///////////// once the socket is opened means once a GPS is connected to the server inside 
///////////// the first loop in here, we'll begin to spawning multi threads for each task 
///////////// comming from the opened GPS socket which is handled by the second loop
///////////// each task might send multiple data through the time to us so we can handle each of them 
///////////// inside the opened GPS socket using spawned tokio threads at least as the size of CPU cores.
/////////////
///////////// EXAMPLE :
/////////////     GPS 1 is connected through our socket server
/////////////     time 1 GPS 1 data sent =>  thread 1 got the job
/////////////     *thread 1 is busy
/////////////     time 2 GPS 1 data sent => thread 2 got the job
/////////////     *thread 1 is done
/////////////     GPS 2 is connected through our socket server
/////////////     time 3 GPS 1 data sent => thread 1 got the job
/////////////     *thread 1 and 2 are busy
/////////////     time 4 GPS 1 data sent => thread 3 got job
/////////////     time 5 GPS 2 data sent => thread 1 got the job
///////////// ============================================================================================================



mod utils;
mod handlers;
mod schemas;
mod macros;
use crate::utils::tcp_controller;
use crate::utils::udp_controller;
use crate::handlers::db::cass::establish as cass;
use crate::handlers::db::pg::establish as pg;
use crate::schemas::device::GPSData;
use std::env;
use std::net::SocketAddr;
use dotenv::dotenv;
use device; //-- to use all functions and codes inside lib.rs




#[tokio::main] //-- await is only allowd inside an async function due to this reason we're using the tokio as a runtime to make the main() function as an async one
async fn main() -> std::io::Result<()>{


        dotenv().expect("?????? .env file not found");
        let socket_type = env::var("SOCKET_TYPE").expect("?????? no socket type set");
        let secret_key = env::var("SECRET_KEY").expect("?????? found no secret key"); // TODO - 
        let environment = env::var("ENVIRONMENT").expect("?????? no environment variable set"); // TODO - 
        let buffer_size = env::var("MAX_BUFFER_SIZE").expect("?????? please set maximum buffer size").parse::<usize>().unwrap_or(1024); //-- the default size of buffer is 1024
        let host = env::var("HOST").expect("?????? please set host in .env");
        let port = env::var("DEVICE_PORT").expect("?????? please set port in .env");
        let addr = format!("{}:{}", host, port).parse::<SocketAddr>().expect("?????? cannot parse the socket address"); //-- we can use ? to see the error instead of expect() to force the compiler panic with error
        let cass_session = cass::connection().await.expect("?????? can't establish cassandra connection"); //-- making cassandra pool of connections for selected node before the accepting each socket connection 
        let pg_pool = pg::connection().await.expect("?????? can't establish pg connections"); //-- making postgres pool of connections before the accepting each socket connection  
        GPSData::init(cass_session.clone()).await; //-- it'll create gps_data column family if there is not any
        println!("\n[+] Listening {} on: {}", socket_type, addr);
        
        
        
        match &socket_type as &str{
            "tcp" => {
                match tcp_controller::run_server(addr, buffer_size, cass_session, pg_pool).await{
                    Ok(_) => { // NOTE - we used `_` cause tcp_controller::run_server() function will return () on none std error successful  
                        println!("[+] ======= HANDLING TCP CONTROLLER =======");
                    }, 
                    Err(e) => { // NOTE - we used `e` cause tcp_controller::run_server() function will return () on std error unsuccessful  
                        println!("[!] TCP CONTROLLER IS NOT REACHABLE - {:?}", e);
                    }
                }
            },
            "udp" => {
                let pg_pool = pg_pool.clone(); //-- cloning the immutable postgres pool connections to move it into every async GPS task scope and share between threads
                let cass_session = cass_session.clone(); //-- cloning the immutable cassandra session so we can share its ownership between multiple threads
                match udp_controller::run_server(addr, buffer_size, cass_session, pg_pool).await{
                    Ok(_) => { // NOTE - we used `_` cause udp_controller::run_server function will return () on none std error successful  
                        println!("[+] ======= HANDLING UDP CONTROLLER =======");
                    }, 
                    Err(e) => { // NOTE - we used `e` cause udp_controller::run_server function will return () on std error unsuccessful
                        println!("[!] UDP CONTROLLER IS NOT REACHABLE - {:?}", e);
                    }
                };
            },
            _ => {
                println!("?????? not supported socket type!");
            }
        }

        Ok(())



}