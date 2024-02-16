--
-- PostgreSQL database dump
--

-- Dumped from database version 15.4 (Debian 15.4-1.pgdg120+1)
-- Dumped by pg_dump version 15.4 (Debian 15.4-1.pgdg120+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: hunttype; Type: TYPE; Schema: public; Owner: p3rtang
--

CREATE TYPE public.hunttype AS ENUM (
    'OldOdds',
    'NewOdds',
    'SOS',
    'DexNav'
);


ALTER TYPE public.hunttype OWNER TO p3rtang;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: p3rtang
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO p3rtang;

--
-- Name: auth_tokens; Type: TABLE; Schema: public; Owner: p3rtang
--

CREATE TABLE public.auth_tokens (
    expire_on timestamp without time zone DEFAULT (now() + '1 day'::interval day) NOT NULL,
    user_uuid uuid NOT NULL,
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL
);


ALTER TABLE public.auth_tokens OWNER TO p3rtang;

--
-- Name: counters; Type: TABLE; Schema: public; Owner: p3rtang
--

CREATE TABLE public.counters (
    name character varying NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    owner_uuid uuid NOT NULL,
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL
);


ALTER TABLE public.counters OWNER TO p3rtang;

--
-- Name: phases; Type: TABLE; Schema: public; Owner: p3rtang
--

CREATE TABLE public.phases (
    name character varying NOT NULL,
    count integer NOT NULL,
    "time" bigint NOT NULL,
    hunt_type public.hunttype NOT NULL,
    has_charm boolean DEFAULT false NOT NULL,
    dexnav_encounters integer,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    owner_uuid uuid NOT NULL,
    parent_uuid uuid NOT NULL,
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL
);


ALTER TABLE public.phases OWNER TO p3rtang;

--
-- Name: preferences; Type: TABLE; Schema: public; Owner: p3rtang
--

CREATE TABLE public.preferences (
    use_default_accent_color boolean DEFAULT true NOT NULL,
    accent_color character varying,
    show_separator boolean DEFAULT false NOT NULL,
    multi_select boolean DEFAULT false NOT NULL,
    user_uuid uuid NOT NULL
);


ALTER TABLE public.preferences OWNER TO p3rtang;

--
-- Name: users; Type: TABLE; Schema: public; Owner: p3rtang
--

CREATE TABLE public.users (
    username character varying NOT NULL,
    password character varying NOT NULL,
    email character varying,
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL
);


ALTER TABLE public.users OWNER TO p3rtang;

--
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
20231013152336	23-10-13 17-23	2024-02-15 09:29:09.876016+00	t	\\xff95923ef9afbcc89efe330243d1754c4719be221420d411481e688ede306676fa9729ff84319a52d2331096e736b892	2870924
20231013152623	23-10-13 17-26	2024-02-15 09:29:09.881911+00	t	\\x6cdafa5fdaa795564b662e68b28a52f04a9836d0b8545316f9bf2b1c3d900030083cf302a822ba5f61d01e4906fd7809	42034765
20231013153846	23-10-13 17-38	2024-02-15 09:29:09.926923+00	t	\\xaca7df4ac0064e8138f0e183701fe01c6a426c0978c334a12947aea0971aff0a672ca9937cd7237b48ee7c9c0a8c701f	2857844
20231020115835	pref multi select	2024-02-15 09:29:09.932734+00	t	\\x5102467c663569c41c958fc8d0a4f3df6718963d76a9fada7338514744b7644d5eaf30fb3240b746d13deb5d17585247	3077303
20240130210737	uuid change	2024-02-15 09:29:09.938738+00	t	\\x2eb7ddac0207eeba0c9bdd83a77b35a55640867a83bab2f0f224940e17bf14dc0fed6f9bc9aa581b0d4132060965f350	53871761
\.


--
-- Data for Name: auth_tokens; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.auth_tokens (expire_on, user_uuid, uuid) FROM stdin;
2024-02-16 09:29:26.310576	2439feb5-c234-4a0c-902b-e5c9e679ffbe	98a58497-6eda-4f01-a8b1-4eaa3359f4d8
2024-02-16 09:30:46.399929	2439feb5-c234-4a0c-902b-e5c9e679ffbe	bf7222d5-e318-4be1-8d81-0c298221ed60
\.


--
-- Data for Name: counters; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.counters (name, created_at, owner_uuid, uuid) FROM stdin;
Counter 1	2024-02-15 09:29:31.983314	2439feb5-c234-4a0c-902b-e5c9e679ffbe	a7602280-e3af-4d03-ac44-c130886cc59b
\.


--
-- Data for Name: phases; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.phases (name, count, "time", hunt_type, has_charm, dexnav_encounters, created_at, owner_uuid, parent_uuid, uuid) FROM stdin;
Phase 1	0	0	NewOdds	f	\N	2024-02-15 09:29:31.993163	2439feb5-c234-4a0c-902b-e5c9e679ffbe	a7602280-e3af-4d03-ac44-c130886cc59b	62cf482d-c5f7-4d43-82e5-74fd38281813
\.


--
-- Data for Name: preferences; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.preferences (use_default_accent_color, accent_color, show_separator, multi_select, user_uuid) FROM stdin;
t	\N	f	f	e86f5900-8e32-4cf9-8d6a-e3e5ed4f7627
f	#abee68	f	f	2439feb5-c234-4a0c-902b-e5c9e679ffbe
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.users (username, password, email, uuid) FROM stdin;
dev	$pbkdf2-sha256$i=100000,l=32$fCoIpWLQg10M2fguLJUgUQ$xQATyGfkb9B5qNw5iPpDNP7zDQxancnr+v+2ykBaCcE	\N	e86f5900-8e32-4cf9-8d6a-e3e5ed4f7627
user	$pbkdf2-sha256$i=100000,l=32$/IvijAkkSqLOVF/T1m/B/A$4ECp85GFTpw3Hu/QYV+dOGNuyvZUtSnNaSkLi6TEQD8	\N	2439feb5-c234-4a0c-902b-e5c9e679ffbe
\.


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: auth_tokens auth_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public.auth_tokens
    ADD CONSTRAINT auth_tokens_pkey PRIMARY KEY (uuid);


--
-- Name: counters counters_pkey; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public.counters
    ADD CONSTRAINT counters_pkey PRIMARY KEY (uuid);


--
-- Name: phases phases_pkey; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public.phases
    ADD CONSTRAINT phases_pkey PRIMARY KEY (uuid);


--
-- Name: preferences preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public.preferences
    ADD CONSTRAINT preferences_pkey PRIMARY KEY (user_uuid);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (uuid);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: p3rtang
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- PostgreSQL database dump complete
--

