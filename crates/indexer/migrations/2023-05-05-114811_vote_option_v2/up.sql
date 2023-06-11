-- Your SQL goes here

CREATE TABLE vote_options(
    id UUID NOT NULL PRIMARY KEY,
    option TEXT NOT NULL,
    vote_weight BIGINT NOT NULL,
    option_elected BOOLEAN NOT NULL,
    proposal_id UUID NOT NULL REFERENCES proposal(id),
    CONSTRAINT vo_unique UNIQUE(proposal_id,option)
);
