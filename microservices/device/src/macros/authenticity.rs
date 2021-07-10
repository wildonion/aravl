





#[macro_export]
macro_rules! authenticity {
    /*
        the rules for api!("url" => "http://localhost:7366/avl/api/auth", "body" => "{}", "headers" => {}) is the following
        $($($name:expr => $key:expr),*)
    */
    ($body:expr, $headers:expr) => {
        {
            match reqwest::Client::builder().build(){ //-- calling api of auth microservice to check the device authenticity
                Ok(client) => {
                    match client
                        .post("http://localhost:7366/avl/api/auth/device")
                        .headers($headers)
                        .body($body)
                        .send()
                        .await{ // NOTE - client future won't solve unless we await on it and once it gets solved we'll fall into the following arms
                            Ok(res) => {
                                // match res.text().await{
                                //     // serde_json::to_string() // serialize to json
                                //     // serde_json::from_str() // deserialize to struct
                                // }
                                match res.json::<GPSConfig>().await{
                                    Ok(json) => {
                                        println!("\t[+] CURRENT SERVER TIME : {:?} | STATUS : ✅ | RESPONSE DATA FROM THE AUTH MICROSERVICE SERVER : {:?}", chrono::Local::now().naive_local(), json);
                                        Ok(json) //-- returning the GPS configurations structure
                                    },
                                    Err(e) => {
                                        println!("\t[!] CURRENT SERVER TIME : {:?} | STATUS : ✅ | CAN'T DESERIALIZE JSON INTO THE GPSConfig STRUCT : {:?}", chrono::Local::now().naive_local(), e);
                                        Err(e)
                                    }
                                }
                            },
                            Err(e) => {
                                println!("\t[!] CURRENT SERVER TIME : {:?} | STATUS : ✅ | AUTH MICROSERVICE SERVER STATUS : {:?}", chrono::Local::now().naive_local(), e);
                                Err(e)
                            }
                        }
                },
                Err(e) => {
                    println!("\t[!] CURRENT SERVER TIME : {:?} | STATUS : ✅ | HTTP CLIENT OBJECT STATUS : {:?}", chrono::Local::now().naive_local(), e);
                    Err(e)
                }
            }
            
        }
    };
}

