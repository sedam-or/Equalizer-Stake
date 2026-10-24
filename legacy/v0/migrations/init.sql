-- Equalizer-Stake: Initial Schema Extensions
-- This file is mounted into postgres and runs on first container start.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";
