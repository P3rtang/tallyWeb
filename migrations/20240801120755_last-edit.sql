-- Add migration script here
ALTER TABLE counters
ADD COLUMN last_edit TIMESTAMP DEFAULT now() NOT NULL;

ALTER TABLE phases
ADD COLUMN last_edit TIMESTAMP DEFAULT now() NOT NULL;
