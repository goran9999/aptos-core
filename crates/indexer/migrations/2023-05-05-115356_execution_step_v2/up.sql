-- Your SQL goes here


CREATE TABLE execution_step (
    id UUID NOT NULL,
    execution_hash TEXT NOT NULL,
    execution_parameters TEXT NOT NULL,
    execution_paramter_types TEXT NOT NULL,
    executed BOOLEAN NOT NULL,
    vote_option_id UUID NOT NULL REFERENCES vote_options(id),
    PRIMARY KEY (id)
);
