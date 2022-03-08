-- Your SQL goes here
create table tags
(
    id   INTEGER   not null
        primary key autoincrement
        unique,
    name CHAR(256) not null
        unique,
    description VARCHAR not null
);
INSERT INTO tags (id, name, description) VALUES (1, 'Rocket', 'Rocket is framework to develop web applications in Rust.');
INSERT INTO tags (id, name, description) VALUES (2, 'Diesel', 'Diesel is database middleware for Rust. It supports multiple backends like SQLite, MySQL and Postgres. ');
INSERT INTO tags (id, name, description) VALUES (3, 'Handlebars', 'Handlebars is a templating engine primarily developed for generating HTML markups dynamically.');