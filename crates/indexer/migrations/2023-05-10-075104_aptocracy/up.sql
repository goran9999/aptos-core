-- Your SQL goes here

CREATE TABLE organization (
    address TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    creator TEXT NOT NULL,
    default_role TEXT NOT NULL,
    governing_coin TEXT NOT NULL,
    governing_collection_info TEXT NOT NULL,
    invite_only BOOLEAN NOT NULL,
    main_governance BIGINT,
    max_voter_weight BIGINT,
    org_type TEXT NOT NULL,
    treasury_count INTEGER NOT NULL,
    role_config TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);