CREATE TABLE users(
    user_id uuid NOT NULL,
    PRIMARY KEY(user_id),
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);