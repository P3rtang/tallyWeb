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
    time        bigint          NOT NULL,
    hunt_type   hunttype        NOT NULL,
    has_charm   bool            NOT NULL DEFAULT false
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
    use_default_accent_color    boolean         NOT NULL DEFAULT true,
    accent_color                varchar,
    show_separator              boolean         NOT NULL DEFAULT false
)
