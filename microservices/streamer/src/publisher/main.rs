





/*

    we can’t just pass the receiver between multiple threads cause trait Copy is not implemented for the receiver thus we can’t clone it to fix the issue cause if a type is Copy its Clone needs to return *self.
    Multiple producer means multiple threads own the receiver and single consumer means only one of them can mutate and get the job or task from the receiver at a time.
    to fix the issue we have to take an atomic reference from the receiver using Arc in order to clone it for passing between multiple threads and for mutating it we have to 
    put it inside a Mutex to insure that only one thread can change the content of the receiver at a time. this is done by waiting on the receiver until a job or task becomes 
    available to the down side of the channel then locking on the receiver to acquire the mutex.
    the receiver of tokio mpsc channel is shareable between tokio::spawn() threads so we don’t need to take an atomic reference and put it inside the Mutex.

*/




mod handlers;
use std::env;
use dotenv::dotenv;






#[tokio::main] //-- await is only allowd inside an async function due to this reason we're using the tokio as a runtime to make the main() function as an async one
async fn main() -> std::io::Result<()>{
    
    

    
    env::set_var("RUST_LOG", "librdkafka=trace,rdkafka::client=debug");
    env_logger::init();
    dotenv().expect("⚠️ .env file not found");
    let host = env::var("KAFKA_HOST").expect("⚠️ please set host in .env");
    let secret_key = env::var("JWT_SECRET_KEY").expect("⚠️ found no jwt secret key"); // TODO - 
    let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set"); // TODO -
    let node1_port = env::var("KAFKA_NODE1_PORT").expect("⚠️ please set kafka node1 port in .env"); //-- broker 1
    let node2_port = env::var("KAFKA_NODE2_PORT").expect("⚠️ please set kafka node2 port in .env"); //-- broker 2
    let node3_port = env::var("KAFKA_NODE3_PORT").expect("⚠️ please set kafka node3 port in .env"); //-- broker 3





    let broker1 = format!("{}:{}", host, node1_port);
    let broker2 = format!("{}:{}", host, node2_port);
    let broker3 = format!("{}:{}", host, node3_port);
    let brokers = format!("{},{},{}", broker1, broker2, broker3);
    handlers::producer::produce(&brokers).await; //-- passing brokers String by taking a reference to it, by doing this we're coercing it into &str - &String is &str


    


    Ok(())



}