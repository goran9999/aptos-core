-- Your SQL goes here


CREATE TABLE treasury (
    treasury_address TEXT NOT NULL,
    aptocracy_address TEXT NOT NULL,
    authority TEXT NOT NULL,
    treasury_index INTEGER NOT NULL,
    deposited_amount BIGINT NOT NULL,
    treasury_coin TEXT NOT NULL,
    governance_id BIGINT NOT NULL,
    PRIMARY KEY (treasury_address)
);