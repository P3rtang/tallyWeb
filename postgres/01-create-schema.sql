create table counters(
    id          serial          NOT NULL,
    user_id     integer         NOT NULL,
    name        character       NOT NULL,
    phases      integer[]       DEFAULT '{}' NOT NULL
);

create table phases(
    id          serial          NOT NULL,
    name        character       NOT NULL,
    count       integer         NOT NULL,
    time        bigint          NOT NULL
);

create table users(
    id          serial          NOT NULL,
    username    character       NOT NULL,
    password    bytea           NOT NULL,
    salt        bytea           NOT NULL
);
