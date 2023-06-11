-- This file should undo anything in `up.sql`
ALTER TABLE organization DROP COLUMN if exists main_treasury;
