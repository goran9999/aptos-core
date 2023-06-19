-- This file should undo anything in `up.sql`
ALTER TABLE proposal DROP COLUMN if exists proposal_type;
