create table if not exists live_streamers
(
    id       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    url      TEXT     not null default '',
    remark   TEXT     not null default ''
);

-- alter table users
--     add constraint users_id_pk primary key (id);
--
-- create index if not exists users_email_idx on users (email);
