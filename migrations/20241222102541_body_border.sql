-- Diff code generated with pgModeler (PostgreSQL Database Modeler)
-- pgModeler version: 1.2.0-alpha1
-- Diff date: 2024-12-22 11:27:48
-- Source model: tally_web
-- Database: tally_web
-- PostgreSQL version: 17.0

-- [ Diff summary ]
-- Dropped objects: 0
-- Created objects: 1
-- Changed objects: 0

SET search_path=public,pg_catalog;
-- ddl-end --


-- [ Created objects ] --
-- object: show_body_border | type: COLUMN --
ALTER TABLE public.preferences ADD COLUMN show_body_border boolean NOT NULL DEFAULT true;
-- ddl-end --


