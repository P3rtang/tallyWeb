COPY public.users (username, password, email, uuid) FROM stdin;
dev	$pbkdf2-sha256$i=100000,l=32$fCoIpWLQg10M2fguLJUgUQ$xQATyGfkb9B5qNw5iPpDNP7zDQxancnr+v+2ykBaCcE	\N	e86f5900-8e32-4cf9-8d6a-e3e5ed4f7627
user	$pbkdf2-sha256$i=100000,l=32$/IvijAkkSqLOVF/T1m/B/A$4ECp85GFTpw3Hu/QYV+dOGNuyvZUtSnNaSkLi6TEQD8	\N	2439feb5-c234-4a0c-902b-e5c9e679ffbe
\.

COPY public.preferences (use_default_accent_color, accent_color, show_separator, multi_select, user_uuid) FROM stdin;
t	\N	f	f	e86f5900-8e32-4cf9-8d6a-e3e5ed4f7627
\.
