-- =====================================================
-- EventAck PostgreSQL Bootstrap for Production / K8s
-- Idempotent and secure
-- Must be run as postgres superuser
-- =====================================================

-- 1️⃣ Create role if it does not exist
DO
$$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_roles WHERE rolname = 'eventack'
    ) THEN
        CREATE ROLE eventack
            LOGIN
            PASSWORD 'eventack'; -- In production, replace with a secret
    END IF;
END
$$;

-- 2️⃣ Create database if it does not exist
SELECT 'CREATE DATABASE eventack OWNER eventack'
WHERE NOT EXISTS (
    SELECT FROM pg_database WHERE datname = 'eventack'
)\gexec

-- 3️⃣ Connect to the database
\connect eventack

-- 4️⃣ Create schema dedicated for EventAck
DO
$$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.schemata
        WHERE schema_name = 'eventack'
    ) THEN
        CREATE SCHEMA eventack AUTHORIZATION eventack;
    END IF;
END
$$;

-- 5️⃣ Grant only necessary privileges to the role
GRANT USAGE ON SCHEMA eventack TO eventack;
GRANT CREATE ON SCHEMA eventack TO eventack;
