-- Your SQL goes here

CREATE TABLE proposal (
    id UUID NOT NULL PRIMARY KEY,
    proposal_id BIGINT NOT NULL,
    treasury_address TEXT NOT NULL,
    aptocracy_address TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    discussion_link TEXT NOT NULL,
    creator TEXT NOT NULL,
    max_vote_weight BIGINT NOT NULL,
    cancelled_at BIGINT,
    created_at BIGINT NOT NULL,
    early_tipping BOOLEAN NOT NULL,
    executed_at BIGINT,
    max_voter_options BIGINT NOT NULL,
    max_voting_time BIGINT NOT NULL,
    state INTEGER NOT NULL,
    vote_threshold TEXT NOT NULL,
    voting_finalized_at BIGINT,
    CONSTRAINT proposal_unique UNIQUE(proposal_id,treasury_address)
);
