-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

ALTER TABLE users
ADD COLUMN uuid UUID
PRIMARY KEY DEFAULT uuid_generate_v4();


-- add the owner uuid column to counters
--
ALTER TABLE counters
ADD COLUMN
owner_uuid UUID;

-- set the correct owner
UPDATE counters
SET owner_uuid = uuid FROM users
WHERE users.id = counters.user_id;

-- owner should not be null
ALTER TABLE counters
ALTER COLUMN owner_uuid SET NOT NULL;

-- drop the old user id column
ALTER TABLE counters
DROP COLUMN user_id;


-- add the owner uuid column to phases
--
ALTER TABLE phases
ADD COLUMN
owner_uuid UUID;

-- set the correct owner
UPDATE phases
SET owner_uuid = uuid FROM users
WHERE users.id = phases.user_id;

-- owner should not be null
ALTER TABLE phases
ALTER COLUMN owner_uuid SET NOT NULL;

-- drop the old user id column
ALTER TABLE phases
DROP COLUMN user_id;


-- assign phase to correct parent counter uuid
--
ALTER TABLE counters
ADD COLUMN uuid UUID
PRIMARY KEY DEFAULT uuid_generate_v4();

-- add the parent column to phases
ALTER TABLE phases
ADD COLUMN parent_uuid UUID;

-- assign the parent counter
UPDATE phases
SET parent_uuid = uuid FROM counters
WHERE phases.id = ANY (counters.phases);

-- parent uuid should not be null
ALTER TABLE phases
ALTER COLUMN parent_uuid SET NOT NULL;

-- drop the counters children column
ALTER TABLE counters
DROP COLUMN phases;

-- assing uuid to phase
ALTER TABLE phases
ADD COLUMN uuid UUID
PRIMARY KEY DEFAULT uuid_generate_v4();

-- drop the id columns
ALTER TABLE counters
DROP COLUMN id;
ALTER TABLE phases
DROP COLUMN id;


-- change preference to uuid
--
ALTER TABLE preferences
ADD COLUMN user_uuid UUID PRIMARY KEY;

-- set the correct user uuid
UPDATE preferences
SET user_uuid = uuid FROM users
WHERE users.id = preferences.user_id;

-- drop the old user_id column
ALTER TABLE preferences
DROP COLUMN user_id;


-- change auth_tokens to uuid
--
ALTER TABLE auth_tokens
ADD COLUMN user_uuid UUID;

UPDATE auth_tokens
SET user_uuid = uuid FROM users
WHERE users.id = auth_tokens.user_id;

ALTER TABLE auth_tokens
ALTER COLUMN user_uuid SET NOT NULL;

ALTER TABLE auth_tokens
DROP COLUMN user_id;

ALTER TABLE auth_tokens
ADD COLUMN uuid UUID
PRIMARY KEY DEFAULT uuid_generate_v4();

ALTER TABLE auth_tokens
DROP COLUMN id;


-- drop id columns from users
--
ALTER TABLE users
DROP COLUMN token;

ALTER TABLE users
DROP COLUMN id;
