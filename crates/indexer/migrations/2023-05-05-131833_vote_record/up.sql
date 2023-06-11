-- Your SQL goes here


CREATE TABLE vote_record (
    member_address TEXT NOT NULL,
    proposal_id BIGINT NOT NULL,
    treasury_address TEXT NOT NULL,
    voter_weight BIGINT NOT NULL,
    elected_options TEXT[] NOT NULL,
    voted_at TIMESTAMP NOT NULL,
    PRIMARY KEY (member_address, proposal_id, treasury_address)
);