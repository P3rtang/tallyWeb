create table counters(
    id          serial          NOT NULL,
    user_id     integer         NOT NULL,
    name        varchar         NOT NULL,
    phases      integer[]       DEFAULT '{}' NOT NULL
);

create table phases(
    id          serial          NOT NULL,
    name        varchar         NOT NULL,
    count       integer         NOT NULL,
    time        bigint          NOT NULL
);

create table users(
    id          serial          NOT NULL,
    username    varchar         NOT NULL,
    password    varchar         NOT NULL,
    token       varchar
);

create table auth_tokens(
    id          varchar         NOT NULL,
    user_id     integer         NOT NULL,
    expire_on   timestamp       DEFAULT now() + interval '1' day NOT NULL
)
