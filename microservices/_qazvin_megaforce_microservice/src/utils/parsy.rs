








use std::collections::HashMap;


pub fn param_parser<'a>(data_string: &'a str, first_splitter: &'a str, second_splitter: &'a str) -> Result<HashMap<&'a str, &'a str>, Box<dyn std::error::Error>>{
    
    let mut hash_map_data = HashMap::new();
    let splitted_data: Vec<&str> = data_string.split(first_splitter).collect();
    for param in splitted_data{
        let splitted_param: Vec<&str> = param.split(second_splitter).collect();
        // checking the number of sattelites
        if splitted_param[0] == "NS"{
            if splitted_param[1].is_empty() == false{
                let ns = splitted_param[1].parse::<i8>().unwrap();
                if ns < 4 {continue;}
            } else{
                println!("[!] WARNING ================= UNKNOWN NUMBER OF SATELLITES");
            }
        }
        hash_map_data.insert(splitted_param[0], splitted_param[1]);
    }

    Ok(hash_map_data)

}
