table! {
    users (id) {
        id -> Uuid,
        national_code -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        phone_number -> Varchar,
        mac -> Varchar,
        sex -> Bpchar,
        age -> Int2,
        reg_date -> Timestamp,
    }
}
