-- Add migration script here
ALTER TABLE preferences
ADD COLUMN save_on_pause boolean DEFAULT true NOT NULL;
