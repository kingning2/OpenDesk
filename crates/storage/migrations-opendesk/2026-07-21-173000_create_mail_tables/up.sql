-- opendesk.db: mail templates, accounts, and message history (M2 / CHG-015, CHG-026).

CREATE TABLE IF NOT EXISTS mail_template (
    id                  TEXT PRIMARY KEY NOT NULL,
    name                TEXT NOT NULL,
    template_intent     TEXT NOT NULL,
    subject_template    TEXT NOT NULL,
    body_text_template  TEXT NOT NULL,
    body_html_template  TEXT,
    locale              TEXT,
    is_system           INTEGER NOT NULL DEFAULT 0,
    is_active           INTEGER NOT NULL DEFAULT 1,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mail_template_intent ON mail_template(template_intent, sort_order ASC);

CREATE TABLE IF NOT EXISTS mail_account (
    id                  TEXT PRIMARY KEY NOT NULL,
    label               TEXT NOT NULL,
    from_address        TEXT NOT NULL,
    from_name           TEXT,
    smtp_host           TEXT NOT NULL,
    smtp_port           INTEGER NOT NULL,
    use_tls             INTEGER NOT NULL DEFAULT 1,
    username            TEXT NOT NULL,
    password_ref        TEXT NOT NULL,
    password_value      TEXT NOT NULL,
    imap_host           TEXT,
    imap_port           INTEGER,
    imap_use_tls        INTEGER,
    imap_sync_enabled   INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS mail_message (
    id                  TEXT PRIMARY KEY NOT NULL,
    customer_id         TEXT REFERENCES customer(id) ON DELETE SET NULL,
    template_id         TEXT REFERENCES mail_template(id) ON DELETE SET NULL,
    account_id          TEXT REFERENCES mail_account(id) ON DELETE SET NULL,
    status              TEXT NOT NULL,
    direction           TEXT NOT NULL,
    subject             TEXT NOT NULL,
    body_text           TEXT NOT NULL,
    body_html           TEXT,
    error_message       TEXT,
    sent_at             TEXT,
    received_at         TEXT,
    imap_uid            INTEGER,
    imap_folder         TEXT,
    rfc_message_id      TEXT,
    in_reply_to         TEXT,
    references_header   TEXT,
    is_favorite         INTEGER NOT NULL DEFAULT 0,
    open_tracking_id    TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    UNIQUE(rfc_message_id)
);

CREATE INDEX IF NOT EXISTS idx_mail_message_customer ON mail_message(customer_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_mail_message_account ON mail_message(account_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_mail_message_direction ON mail_message(direction, created_at DESC);

INSERT OR IGNORE INTO mail_template (
    id, name, template_intent, subject_template, body_text_template,
    body_html_template, locale, is_system, is_active, sort_order, created_at, updated_at
) VALUES
(
    'tpl_first_contact_en',
    'First Contact (EN)',
    'first_contact',
    'Partnership with {{customer.display_name}}',
    'Hi {{customer.display_name}},\n\nI found your channel {{customer.source_title}} and would like to discuss a collaboration opportunity.\n\nBest,\n{{sender.name}}\n{{sender.email}}',
    NULL,
    'en',
    1,
    1,
    1,
    '1753090000000',
    '1753090000000'
),
(
    'tpl_follow_up_en',
    'Follow Up (EN)',
    'follow_up',
    'Following up on our last note',
    'Hi {{customer.display_name}},\n\nFollowing up on my previous email regarding collaboration.\n\nBest,\n{{sender.name}}\n{{sender.email}}',
    NULL,
    'en',
    1,
    1,
    2,
    '1753090000000',
    '1753090000000'
),
(
    'tpl_quote_proposal_en',
    'Quote Proposal (EN)',
    'quote_proposal',
    'Quote proposal for {{customer.display_name}}',
    'Hi {{customer.display_name}},\n\nBased on your profile, our current proposal is {{customer.quoted_price}} {{customer.currency}} for the {{customer.package_name}} package.\n\nBest,\n{{sender.name}}\n{{sender.email}}',
    NULL,
    'en',
    1,
    1,
    3,
    '1753090000000',
    '1753090000000'
),
(
    'tpl_quote_revision_en',
    'Quote Revision (EN)',
    'quote_revision',
    'Updated proposal for {{customer.display_name}}',
    'Hi {{customer.display_name}},\n\nI updated the proposal to {{customer.quoted_price}} {{customer.currency}}. Let me know what you think.\n\nBest,\n{{sender.name}}\n{{sender.email}}',
    NULL,
    'en',
    1,
    1,
    4,
    '1753090000000',
    '1753090000000'
),
(
    'tpl_cooperation_confirm_en',
    'Cooperation Confirm (EN)',
    'cooperation_confirm',
    'Great to work with you',
    'Hi {{customer.display_name}},\n\nHappy to confirm the cooperation for {{customer.package_name}} starting {{today.date}}.\n\nBest,\n{{sender.name}}\n{{sender.email}}',
    NULL,
    'en',
    1,
    1,
    5,
    '1753090000000',
    '1753090000000'
);
