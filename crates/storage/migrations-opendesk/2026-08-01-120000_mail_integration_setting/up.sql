-- opendesk.db: configurable external mail integrations (email-read API, etc.).

CREATE TABLE IF NOT EXISTS mail_integration_setting (
    id                      TEXT PRIMARY KEY NOT NULL,
    enabled                 INTEGER NOT NULL DEFAULT 0,
    api_base                TEXT NOT NULL DEFAULT '',
    pixel_path_template     TEXT NOT NULL DEFAULT '',
    query_path_template     TEXT NOT NULL DEFAULT '',
    parse_script            TEXT NOT NULL DEFAULT '',
    updated_at              TEXT NOT NULL
);

INSERT OR IGNORE INTO mail_integration_setting (
    id,
    enabled,
    api_base,
    pixel_path_template,
    query_path_template,
    parse_script,
    updated_at
) VALUES (
    'email_read',
    1,
    'https://kol-service.gbyte.com',
    '/api/v1/email-read/pixel?email={{email}}&mailId={{mailId}}',
    '/api/v1/email-read?email={{email}}&mailId={{mailId}}',
    'function parseResponse(data) {\n  const items = data.items || [];\n  const openedAt = items[0]?.opened_at || items[0]?.open_time || items[0]?.timestamp || null;\n  return { openCount: items.length, openedAt };\n}',
    '1754000000000'
);
