DROP DATABASE IF Exists tally_web;
DROP USER IF EXISTS p3rtang;
CREATE USER p3rtang PASSWORD 'ktkNfiGEW4tr8T';
ALTER USER p3rtang CREATEDB;
CREATE DATABASE tally_web OWNER p3rtang ENCODING = 'UTF-8';
