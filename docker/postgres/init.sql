-- Enable extensions for the opennvr database
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create keycloak database on the same PostgreSQL instance
CREATE DATABASE keycloak;
