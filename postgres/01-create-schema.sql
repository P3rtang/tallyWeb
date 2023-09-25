create table counters(
    id          serial          UNIQUE NOT NULL,
    user_id     integer         NOT NULL,
    name        varchar         NOT NULL,
    phases      integer[]       DEFAULT '{}' NOT NULL
);

create table phases(
    id          serial          UNIQUE NOT NULL,
    user_id     integer         NOT NULL,
    name        varchar         NOT NULL,
    count       integer         NOT NULL,
    time        bigint          NOT NULL
);

create table users(
    id          serial          UNIQUE NOT NULL,
    username    varchar         UNIQUE NOT NULL,
    password    varchar         NOT NULL,
    token       varchar
);

create table auth_tokens(
    id          varchar         UNIQUE NOT NULL,
    user_id     integer         UNIQUE NOT NULL,
    expire_on   timestamp       DEFAULT now() + interval '1' day NOT NULL
);

create table preferences(
    user_id                     integer         UNIQUE NOT NULL,
    use_default_accent_color    boolean         DEFAULT true NOT NULL,
    accent_color                varchar
)
