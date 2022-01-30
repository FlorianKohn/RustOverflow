-- Your SQL goes here
create table chosen_tags
(
    id       INTEGER not null
        primary key autoincrement
        unique,
    question INTEGER not null
        references questions (id),
    tag      INTEGER not null
        references tags (id)
);