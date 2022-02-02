-- Your SQL goes here
create table users
(
    id       INTEGER   not null
        primary key autoincrement
        unique,
    username CHAR(256) not null unique,
    password CHAR(60)  not null
);