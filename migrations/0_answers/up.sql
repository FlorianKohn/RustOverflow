-- Your SQL goes here
CREATE TABLE answers (
     id       INTEGER  PRIMARY KEY AUTOINCREMENT
         UNIQUE
                       NOT NULL,
     author   INTEGER  REFERENCES users (id)
                       NOT NULL,
     question INTEGER  REFERENCES questions (id)
                       NOT NULL,
     time     DATETIME NOT NULL
         DEFAULT ( (datetime('now', 'localtime') ) ),
     score    INTEGER  NOT NULL
         DEFAULT (0),
     accepted BOOLEAN  NOT NULL
         DEFAULT (FALSE),
     text     VARCHAR  NOT NULL
);
