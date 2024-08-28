-- Add migration script here
ALTER TABLE counters
ADD COLUMN is_deleted boolean DEFAULT false NOT NULL;

ALTER TABLE phases
ADD COLUMN is_deleted boolean DEFAULT false NOT NULL;
