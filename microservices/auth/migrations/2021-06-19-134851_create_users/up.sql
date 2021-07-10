-- Your SQL goes here


CREATE TABLE users(
    id UUID NOT NULL PRIMARY KEY,
    national_code VARCHAR NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    phone_number VARCHAR NOT NULL,
    mac VARCHAR NOT NULL,
    sex CHAR(1) DEFAULT 'n' NOT NULL,
    age SMALLINT DEFAULT 0 NOT NULL,
    reg_date TIMESTAMP NOT NULL
)
