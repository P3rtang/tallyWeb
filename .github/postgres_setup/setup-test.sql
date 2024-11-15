--
-- PostgreSQL database dump
--

-- Dumped from database version 15.4 (Debian 15.4-1.pgdg120+1)
-- Dumped by pg_dump version 16.3

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
-- Data for Name: counters; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.counters (name, created_at, owner_uuid, uuid) FROM stdin;
Counter 1	2024-02-15 09:29:31.983314	2439feb5-c234-4a0c-902b-e5c9e679ffbe	8d0604b5-6be2-4346-a03c-c8fa4f8bb6b9
\.


--
-- Data for Name: phases; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.phases (name, count, "time", hunt_type, has_charm, dexnav_encounters, created_at, owner_uuid, parent_uuid, uuid, success) FROM stdin;
Phase 1	0	0	NewOdds	f	\N	2024-02-15 09:29:31.993163	2439feb5-c234-4a0c-902b-e5c9e679ffbe	8d0604b5-6be2-4346-a03c-c8fa4f8bb6b9	62cf482d-c5f7-4d43-82e5-74fd38281813	f
\.


--
-- Data for Name: preferences; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.preferences (use_default_accent_color, accent_color, show_separator, multi_select, user_uuid) FROM stdin;
t	\N	f	f	2439feb5-c234-4a0c-902b-e5c9e679ffbe
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: p3rtang
--

COPY public.users (username, password, email, uuid) FROM stdin;
user	$pbkdf2-sha256$i=100000,l=32$/IvijAkkSqLOVF/T1m/B/A$4ECp85GFTpw3Hu/QYV+dOGNuyvZUtSnNaSkLi6TEQD8	\N	2439feb5-c234-4a0c-902b-e5c9e679ffbe
\.


--
-- PostgreSQL database dump complete
--

