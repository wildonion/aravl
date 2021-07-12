







////////////////////////////////////////// TCP MODULE //////////////////////////////////////////
pub mod tcp_controller{
    use crate::handlers::{tcp::process_connection as tcp_handler};
    use tokio::net::TcpListener;
    use std::net::SocketAddr;
    use std::sync::Arc;
    pub async fn run_server(addr: SocketAddr, buffer_size: usize) -> Result<(), Box<dyn std::error::Error>>{ //-- the actual type of the object that implements Error trait only becomes known at runtime and because of that we've used Box<dyn Trait> cause it's a trait object
        let tcp_listener = TcpListener::bind(&addr).await?;
        loop{ //-- loop through all incoming requests
            let (mut socket, device_addr) = tcp_listener.accept().await?;
            println!("\n[+] CONNECTION establishED FROM GPS : {} AT TIME : {:?}", device_addr, chrono::Local::now().naive_local());
            //-- adding each incoming task to the channel queue then spawning multi threads per GPS data or the task which is inside the channel queue in its socket
            //-- we use async move {} block to move all the env vars before the task into the async block to have lifetime across .await
            //-- a new task is spawned for each inbound socket, the socket is moved to the new task and processed there
            tokio::spawn(async move{ //-- tokio spawns multiple threads inside its reactor runtime to execute the task inside a free thread - each thread has a loop {} to listen constantly on every new task to get that 
                let mut buffer = vec![0u8; buffer_size]; //-- allocating 0 bytes of u8 type for the buffer on the stack for every incoming task or job or data from the connected GPS socket - Vec<u8> is coerced later to &[u8]
                tcp_handler(socket, buffer, device_addr).await; //-- ? can not be applied to (), means process_connection returns () which literally returns nothing so we can't unwrap it! - when we are awaiting on the process_connection future all data types inside the process_connection function must have a valid lifetime across the whole await means their lifetime must be valid until the future is resolved
            });
        }
    }
}
