-- Diff code generated with pgModeler (PostgreSQL Database Modeler)
-- pgModeler version: 1.2.0-alpha1
-- Diff date: 2024-11-15 17:46:51
-- Source model: tally_web
-- Database: tally_web
-- PostgreSQL version: 17.0

-- [ Diff summary ]
-- Dropped objects: 0
-- Created objects: 7
-- Changed objects: 0

SET search_path=public,pg_catalog;
-- ddl-end --


-- [ Created objects ] --
-- object: step_size | type: COLUMN --
ALTER TABLE public.phases ADD COLUMN step_size integer NOT NULL DEFAULT 1;
-- ddl-end --

COMMENT ON COLUMN public.phases.step_size IS E'Step size indicates the amount needed to add to count, when an add_count command is given in the `CountableStore`.';
-- ddl-end --




-- [ Created constraints ] --
-- object: phases_primary_key | type: CONSTRAINT --
ALTER TABLE public.phases DROP CONSTRAINT IF EXISTS phases_pkey CASCADE;
ALTER TABLE public.phases ADD CONSTRAINT phases_primary_key PRIMARY KEY (uuid);
-- ddl-end --



-- [ Created foreign keys ] --
-- object: owner_foreign_key | type: CONSTRAINT --
ALTER TABLE public.phases DROP CONSTRAINT IF EXISTS owner_foreign_key CASCADE;
ALTER TABLE public.phases ADD CONSTRAINT owner_foreign_key FOREIGN KEY (owner_uuid)
REFERENCES public.users (uuid) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: parent_foreign_key | type: CONSTRAINT --
ALTER TABLE public.phases DROP CONSTRAINT IF EXISTS parent_foreign_key CASCADE;
ALTER TABLE public.phases ADD CONSTRAINT parent_foreign_key FOREIGN KEY (parent_uuid)
REFERENCES public.counters (uuid) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: user_foreign_key | type: CONSTRAINT --
ALTER TABLE public.preferences DROP CONSTRAINT IF EXISTS user_foreign_key CASCADE;
ALTER TABLE public.preferences ADD CONSTRAINT user_foreign_key FOREIGN KEY (user_uuid)
REFERENCES public.users (uuid) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: user_foreign_key | type: CONSTRAINT --
ALTER TABLE public.auth_tokens DROP CONSTRAINT IF EXISTS user_foreign_key CASCADE;
ALTER TABLE public.auth_tokens ADD CONSTRAINT user_foreign_key FOREIGN KEY (user_uuid)
REFERENCES public.users (uuid) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: owner_foreign_key | type: CONSTRAINT --
ALTER TABLE public.counters DROP CONSTRAINT IF EXISTS owner_foreign_key CASCADE;
ALTER TABLE public.counters ADD CONSTRAINT owner_foreign_key FOREIGN KEY (owner_uuid)
REFERENCES public.users (uuid) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

