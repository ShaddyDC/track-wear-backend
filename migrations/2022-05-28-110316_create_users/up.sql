CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    sub VARCHAR NOT NULL UNIQUE,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL
);