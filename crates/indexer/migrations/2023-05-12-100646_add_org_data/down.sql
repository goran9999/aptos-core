-- This file should undo anything in `up.sql`
ALTER TABLE organization DROP COLUMN if exists image;
ALTER TABLE organization ADD COLUMN if exists description;