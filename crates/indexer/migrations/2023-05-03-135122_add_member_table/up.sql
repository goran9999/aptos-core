-- Your SQL goes here
CREATE TABLE member (
    member_address VARCHAR(66) NOT NULL,
    aptocracy_address VARCHAR(66) NOT NULL,
    role TEXT NOT NULL,
    status BIGINT,
    proposal_created BIGINT,
    -- Constraints
    PRIMARY KEY (
    member_address,
    aptocracy_address
  )
);