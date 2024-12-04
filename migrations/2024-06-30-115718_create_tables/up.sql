CREATE TABLE IF NOT EXISTS transactions (
    transaction_hash TEXT PRIMARY KEY,
    transaction_status BOOLEAN NOT NULL,
    block_hash TEXT NOT NULL,
    block_number BIGINT NOT NULL,
    "from" TEXT NOT NULL,
    "to" TEXT,
    contract_address TEXT,
    logs_count BIGINT NOT NULL,
    input TEXT NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users_searches (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    FOREIGN KEY (transaction_hash) REFERENCES transactions(transaction_hash)
);

