-- Your SQL goes here
create table tags
(
    id   INTEGER   not null
        primary key autoincrement
        unique,
    name CHAR(256) not null
        unique
);