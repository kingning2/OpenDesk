CREATE TABLE channel_accounts (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    credential TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE channel_conversations (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    peer_id TEXT NOT NULL,
    peer_name TEXT,
    item_id TEXT,
    item_title TEXT,
    item_price INTEGER,
    updated_at TEXT NOT NULL
);

CREATE TABLE channel_messages (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_channel_messages_conversation ON channel_messages(conversation_id, created_at);

CREATE TABLE channel_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
