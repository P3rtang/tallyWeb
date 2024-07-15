REVOKE CONNECT ON DATABASE tally_web FROM public;

SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE datname = current_database()
  AND pid <> pg_backend_pid();

DROP DATABASE IF Exists tallyweb;
DROP USER IF EXISTS p3rtang;
CREATE USER p3rtang PASSWORD 'ktkNfiGEW4tr8T';
ALTER USER p3rtang CREATEDB;
CREATE DATABASE tally_web OWNER p3rtang ENCODING = 'UTF-8';
