-- Add migration script here
alter table preferences add column multi_select bool default false not null;
