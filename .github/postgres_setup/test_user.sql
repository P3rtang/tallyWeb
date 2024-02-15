COPY public.users (username, password, email, uuid) FROM stdin;
dev	$pbkdf2-sha256$i=100000,l=32$fCoIpWLQg10M2fguLJUgUQ$xQATyGfkb9B5qNw5iPpDNP7zDQxancnr+v+2ykBaCcE	\N	e86f5900-8e32-4cf9-8d6a-e3e5ed4f7627
user	$pbkdf2-sha256$i=100000,l=32$/IvijAkkSqLOVF/T1m/B/A$4ECp85GFTpw3Hu/QYV+dOGNuyvZUtSnNaSkLi6TEQD8	\N	2439feb5-c234-4a0c-902b-e5c9e679ffbe
\.

COPY public.preferences (use_default_accent_color, accent_color, show_separator, multi_select, user_uuid) FROM stdin;
t	\N	f	f	e86f5900-8e32-4cf9-8d6a-e3e5ed4f7627
\.

COPY public.counters (name, created_at, owner_uuid, uuid) FROM stdin;
Counter 1	2024-02-15 09:29:31.983314	2439feb5-c234-4a0c-902b-e5c9e679ffbe	a7602280-e3af-4d03-ac44-c130886cc59b
\.

COPY public.phases (name, count, "time", hunt_type, has_charm, dexnav_encounters, created_at, owner_uuid, parent_uuid, uuid) FROM stdin;
Phase 1	0	0	NewOdds	f	\N	2024-02-15 09:29:31.993163	2439feb5-c234-4a0c-902b-e5c9e679ffbe	a7602280-e3af-4d03-ac44-c130886cc59b	62cf482d-c5f7-4d43-82e5-74fd38281813
\.
