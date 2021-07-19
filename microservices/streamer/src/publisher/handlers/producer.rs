




/*

    implementing futures based on multithreading mpsc and tokio async approach using rdkafka lib
    rdkafka is a asynchronous task and messaging handler with more control on heavy event streaming and memeory management 
    based on kafka and tokio multithreading runtime so it solves and sends all events future of our topic in an asynchronous manner 
    inside tokio threads using mpsc protocol by awaiting on each message or event.
    tokio::spawn() is a multithreaded async task handler

    we can use raw future messages of rust by awaiting on them using tokio runtime 
    or use tokio::spawn() which is multithreaded asynchronous task handler 
    to solve every event future sent by the producer.
    
    
    we can’t just pass the receiver between multiple threads cause trait Copy is not implemented for the receiver thus we can’t clone it to fix the issue cause if a type is Copy its Clone needs to return *self.
    Multiple producer means multiple threads own the receiver and single consumer means only one of them can mutate and get the job or task from the receiver at a time.
    to fix the issue we have to take an atomic reference from the receiver using Arc in order to clone it for passing between multiple threads and for mutating it we have to 
    put it inside a Mutex to insure that only one thread can change the content of the receiver at a time. this is done by waiting on the receiver until a job or task becomes 
    available to the down side of the channel then locking on the receiver to acquire the mutex.
    the receiver of tokio mpsc channel is shareable between tokio::spawn() threads so we don’t need to take an atomic reference and put it inside the Mutex.

*/



use crate::handlers::db::cass::schemas::device::GPSData;
use crate::handlers::db::cass::establish as cass;
use crate::handlers::db::pg::establish as pg;
use std::time::SystemTime;
use rdkafka::config::ClientConfig;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{FutureProducer, FutureRecord};



pub async fn produce(brokers: &str){



    let pg_pool = pg::connection().await.expect("⚠️ can't establish pg connections"); //-- making postgres pool of connections before the accepting each socket connection
    let cass_session = cass::connection().await.expect("⚠️ can't establish cassandra connection!"); //-- making cassandra pool of connections for selected node
    let producer: &FutureProducer = &ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("⚠️ producer creation error");
    
    
    
    
    
    
    let pg_pool = pg_pool.clone(); //-- cloning the immutable postgres pool connections to move it into every async task scope and share between tokio::spawn() threads
    let cass_session = cass_session.clone(); //-- cloning the immutable cassandra session so we can share its ownership between multiple threads
    let producer = producer.clone(); //-- we're clonning the producer cause we want to move it between tokio::spawn() threads thus according to rust ownership we have to take a reference to the producer using clone() cause trait Copy is not imeplemented for that
    tokio::spawn(async move{
        let mut i = 0_usize;
        loop {
            // TODO - fetching rows from some table
            // ...
            // let mut conn = pg_pool.get().unwrap();
            // let some_id_for_fetch = 23;
            // let fetch_statement = conn.prepare("SELECT some_column FROM some_table WHERE some_thing = $1").unwrap();
            // let rows = conn.query(&fetch_statement, &[&some_id_for_fetch]).unwrap();
            // let device_id: String = rows[0].get(0); //-- getting the first row of the first column
            // TODO - https://itnext.io/getting-started-with-kafka-and-rust-part-1-e0074961ec6b
            // TODO - publish cassandra GPS data every n seconds using kafka streamer which is a multithreaded async task handler with mpsc or mpmc job queue channel protocol
            // TODO - send the last GPS data event in every iteration based on its imei topic, so we must have separate topic for each activated device based on their imei
            let device_event = GPSData::last(cass_session.clone()).await; //-- getting the last data inserted into cassandra gps_data column family
            let topic = device_event.imei.to_string(); //-- getting its imei to set it as the topic for this event
            let device_event_json = serde_json::to_string_pretty(&device_event).expect("⚠️ failed to serialize device event"); //-- serializing the struct into json
            let key = &i.to_string(); //-- setting the key for this event
            let devlivery_status = producer.send_result( //-- we're using FutureRecord for sending the message or the event asynchoronously to all consumers cause send_result() method takes a FutureRecord to send a message
            FutureRecord::to(&topic)
                        .key(key)
                        .payload(&device_event_json) //-- we can send serde json inside the payload
                        .headers(OwnedHeaders::new().add("wo_header_key", "wo_header_value"))
                        .timestamp(
                            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH){
                                Ok(n) =>  n.as_secs() as i64,
                                Err(_) => { panic!("SystemTime before UNIX EPOCH!") },
                            }
                        )
            );
            println!("[+] Delivery status for GPS data with imei {} inside iteration key {} received", topic, i);
            match devlivery_status{ //-- devlivery_status is a Result of delivery future and in order to solve it we have to await on it 
                Ok(delivery) => {
                    let solved_delivery = delivery.await.unwrap().unwrap();
                    println!("[+] Delivery solved {:?}", solved_delivery);
                },
                Err(e) => {
                    println!("[!] Delivery error {:?}", e);
                }
            }

            i += 1;
        }
    });





            
    



    // let topic = "device imei";
    // let fututres = (0..5)
    //     .map(|i| async move { //-- async closure for sending 5 messages or events, each with unique key
    //         let cass_session = cass::connection().await.expect("⚠️ can't establish cassandra connection!"); //-- making cassandra pool of connections for selected node
    //         let device_event = GPSData::last(cass_session.clone()).await;
    //         let device_event_json = serde_json::to_string_pretty(&device_event).expect("⚠️ failed to serialize device event");
    //         let delivery_status = producer
    //             .send(
    //                 FutureRecord::to(topic)
    //                             .payload(&format!(">>> Event - {} contains {}", i, device_event_json)) //-- we can send serde json inside the payload
    //                             .key(&format!(">>> Event key {}", i))
    //                             .headers(OwnedHeaders::new().add("wo_header_key", "wo_header_value")),
    //           Duration::from_secs(0),
    //             ).await;
    //         println!("[+] Delivery status for event {} received", i);
    //         delivery_status
    //     }).collect::<Vec<_>>(); //-- here _ means the type of collected data which is 5 Futures in here ( impl Future<Output = ()> )
    //     for future in fututres{
    //         println!("[+] Future completed. Future info => {:?}", future.await);
    //     }







}
