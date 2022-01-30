-- Your SQL goes here
create table questions
(
    id     INTEGER not null
        primary key autoincrement
        unique,
    author INTEGER not null
        references users (id)
            on delete restrict,
    time   DATETIME default (datetime('now', 'localtime')) not null,
    score  INTEGER  default 0 not null,
    title  VARCHAR not null,
    text   VARCHAR not null
);