-- 业务数据通用记录表：各业务 Store 的 JSON 序列化持久化。
-- domain 区分业务域（account / keyword / order / item / card / blacklist /
-- filter / notification / risk / setting / feedback / publish_material /
-- publish_log / address / batch / auto_reply_log 等），record_id 为域内主键。
CREATE TABLE business_records (
  domain    TEXT    NOT NULL,
  record_id TEXT    NOT NULL,
  owner_id  INTEGER NOT NULL DEFAULT 0,
  payload   TEXT    NOT NULL,
  PRIMARY KEY (domain, record_id)
);
CREATE INDEX idx_business_records_domain_owner ON business_records (domain, owner_id);
