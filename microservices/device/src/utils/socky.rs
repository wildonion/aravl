







////////////////////////////////////////// TCP MODULE //////////////////////////////////////////
pub mod tcp_controller{
    use crate::handlers::{tcp::process_connection as tcp_handler};
    use crate::handlers::db::{cass::establish::CassSession, pg::establish::Pool};
    use tokio::net::TcpListener;
    use std::net::SocketAddr;
    use std::sync::Arc;
    pub async fn run_server(addr: SocketAddr, buffer_size: usize, cass_session : Arc<CassSession>, pg_pool: Pool) -> Result<(), Box<dyn std::error::Error>>{ //-- the actual type of the object that implements Error trait only becomes known at runtime and because of that we've used Box<dyn Trait> cause it's a trait object
        let tcp_listener = TcpListener::bind(&addr).await?;
        loop{ //-- loop through all incoming requests
            let (mut socket, device_addr) = tcp_listener.accept().await?;
            let pg_pool = pg_pool.clone(); //-- cloning the immutable postgres pool connections to move it into every async GPS task scope and share between threads
            let cass_session = cass_session.clone(); //-- cloning the immutable cassandra session so we can share its ownership between multiple threads
            println!("\n[+] CONNECTION ESTABLISHED FROM GPS : {} AT TIME : {:?}", device_addr, chrono::Local::now().naive_local());
            //-- adding each incoming task to the channel queue then spawning multi threads per GPS data or the task which is inside the channel queue in its socket
            //-- we use async move {} block to move all the env vars before the task into the async block to have lifetime across .await
            //-- a new task is spawned for each inbound socket, the socket is moved to the new task and processed there
            tokio::spawn(async move{ //-- tokio spawns multiple threads inside its reactor runtime to execute the task inside a free thread - each thread has a loop {} to listen constantly on every new task to get that 
                let mut buffer = vec![0u8; buffer_size]; //-- allocating 0 bytes of u8 type for the buffer on the stack for every incoming task or job or data from the connected GPS socket - Vec<u8> is coerced later to &[u8]
                tcp_handler(socket, cass_session, pg_pool, buffer, device_addr).await; //-- ? can not be applied to (), means process_connection returns () which literally returns nothing so we can't unwrap it! - when we are awaiting on the process_connection future all data types inside the process_connection function must have a valid lifetime across the whole await means their lifetime must be valid until the future is resolved
            });
        }
    }
}
////////////////////////////////////////// UDP MODULE ////////////////////////////////////////// 
pub mod udp_controller{
    use crate::handlers::{udp::process_connection as udp_handler};
    use crate::handlers::db::{cass::establish::CassSession, pg::establish::Pool};
    use tokio::sync::mpsc;
    use tokio::net::UdpSocket;
    use std::net::SocketAddr;
    use std::sync::Arc;
    pub async fn run_server(addr: SocketAddr, buffer_size: usize, cass_session: Arc<CassSession>, pg_pool: Pool) -> Result<(), Box<dyn std::error::Error>>{ //-- the actual type of the object that implements Error trait only becomes known at runtime and because of that we've used Box<dyn Trait> cause it's a trait object
        let udp_listener = UdpSocket::bind(&addr).await?;
        let udp_listener_reference = Arc::new(udp_listener); //-- putting our udp socket inside Arc for later cloning
        let socket = udp_listener_reference.clone(); //-- cloning our udp socket to send and share it between threads and tokio::spawn() tasks using mpsc channel protocol 
        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(buffer_size); //-- send vector of u8 bytes and device socket address between tokio tasks (handled by multiple threads) using mpsc channel protocol 
        //-- tokio::spawn() is an async task handler which will solve each task through multiple threads (thread pool) using job/task queue channel protocol like mpsc
        //-- to move data of a task between threads (thread pool) of tokio::spawn() we have to use Arc, note that the none safe Send and Sync trait objects must be implemented for the data
        //-- to mutate data of a task in each tokio::spawn() threads (thread pool) we have to use Mutex
        //-- to move and send shareable data (cloned with Arc) or job/tasks between each tokio tasks' threads we have to use something like mpsc job/task queue channels protocol
        //-- if there was multiple tokio::spawn() we would have to await on each of them
        //-- sending from multiple tasks' threads is done by cloning the sender but we can't share the receiver cause we must have only one consumer
        tokio::spawn(async move{
            udp_handler(rx, socket, cass_session, pg_pool).await; //-- passing receiver and the socket through the udp_handler function to receive buffer and device socket address at the end of the channel
        });
        loop { //-- loop through all incoming requests
            let mut buffer = vec![0u8; buffer_size]; //-- defining a new buffer for each device socket
            match udp_listener_reference.recv_from(&mut buffer).await{ //-- receiving bytes from the socket to fill the buffer

                ///////////// =======================================================================
                ///////////// device is disconnected from our server thus no incoming bytes are exist 
                ///////////// =======================================================================
                Ok((buffer_bytes, device_addr)) if buffer_bytes == 0 => { //-- if the buffer_bytes is 0 means we're about to close the socket 
                    println!("[!] DEVICE : {} | CURRENT SERVER TIME : {:?} | STATUS : ❌", device_addr, chrono::Local::now().naive_local());
                },
                
                ///////////// ========================================
                ///////////// filling buffer with incoming UTF-8 bytes
                ///////////// ========================================
                Ok((buffer_bytes, device_addr)) => {
                    println!("\n[+] CONNECTION ESTABLISHED FROM GPS : {} AT TIME : {:?}", device_addr, chrono::Local::now().naive_local());
                    match tx.send((buffer[0..buffer_bytes].to_vec(), device_addr)).await{
                        Ok(_) => {
                            // https://stackoverflow.com/questions/31012923/what-is-the-difference-between-copy-and-clone
                            //-- sending filled buffer and device socket address through the mpsc channel inside the accepting socket loop
                            // NOTE - we couldn't put the tokio::spawn() in here cause the rx or the receiver will be moved inside the first iteration of the loop
                            // NOTE - we can’t just pass the receiver between multiple threads cause trait Copy is not implemented for the receiver
                            // NOTE - we can't have a clone from the receiver in mpsc protocol to fix the issue cause if a type is Copy it must have Clone also and its Clone needs to return *self
                            // NOTE - can't clone a data structure unless the trait Clone is implemented for that otherwise in order to move it between threads we have to clone it using Arc
                            // NOTE - every Copy type is also required to be Clone and if T: Copy, x: T, and y: &T, then let x = y.clone(); is equivalent to let x = *y;
                            // NOTE - when we derive a Copy implementation Clone is also required cause it's a supertrait of Copy.
                        },
                        Err(e) => {
                            println!("[!] CURRENT SERVER TIME : {:?} | MPSC CHANNEL STATUS : ❓ - {:?}", chrono::Local::now().naive_local(), e);
                        }
                    }
                },
                
                ///////////// ======================
                ///////////// can't read from socket
                ///////////// ======================
                Err(e) => {
                    println!("[!] CURRENT SERVER TIME : {:?} | STATUS : ❓ - {:?}", chrono::Local::now().naive_local(), e);
                }
            }; 
        }
    }
}
