-- Your SQL goes here


CREATE TABLE scripts(
    script_hash TEXT PRIMARY KEY NOT NULL,
    proposal_type INTEGER NOT NULL,
    bytecode bytea
)