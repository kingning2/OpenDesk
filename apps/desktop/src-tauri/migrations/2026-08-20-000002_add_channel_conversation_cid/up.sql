-- 会话表增加 goofish 会话 id（cid），用于消息历史/发送的会话标识。
ALTER TABLE channel_conversations ADD COLUMN cid TEXT;
CREATE INDEX idx_channel_conversations_cid ON channel_conversations(cid);
