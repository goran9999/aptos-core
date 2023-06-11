-- Your SQL goes here
CREATE TABLE governance (
        aptocracy_address VARCHAR(66) NOT NULL,
        governance_id BIGINT NOT NULL,
        max_voting_time BIGINT NOT NULL,
        quorum BIGINT NOT NULL,
        approval_quorum BIGINT NOT NULL,
        early_tipping BOOLEAN NOT NULL,
        valid_from BIGINT NOT NULL,
        valid_to BIGINT,
        --Constraints
        PRIMARY KEY (
            aptocracy_address,
            governance_id
        )
)