-- Add migration script here
ALTER TABLE phases
ADD COLUMN success boolean DEFAULT false NOT NULL
