-- Your SQL goes here

CREATE TABLE deposit_record (
    treasury_address TEXT NOT NULL,
    member_address TEXT NOT NULL,
    aptocracy_address TEXT NOT NULL,
    accumulated_amount BIGINT NOT NULL,
    last_deposit TIMESTAMP NOT NULL,
    PRIMARY KEY (treasury_address, member_address)
);