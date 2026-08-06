# OpenDesk 后端中文文案（Fluent）。
#
# 用于后端直接下发给前端的用户可见文案（IPC / 事件译文）。
# 后续如需英文，新增 en-US.ftl 并在 Translator 中按 locale 加载。

job-queued = 排队中

job-prepare-keywords = 准备爬取 {$keywords_total} 个关键词

job-cancelled = 任务已取消

job-progress = 关键词进度 {$done}/{$total} · 当前「{$keyword}」· 本词收录 {$keyword_accepted} · 合计收录 {$accepted_count}

job-keyword-done = 关键词进度 {$done}/{$total}

job-failed = 失败：{$error}

job-status-unavailable = 状态不可用

stop-keywords-finished = 已完成

stop-max-total-reached = 已达数量上限

stop-quota-exceeded = YouTube 配额已用尽，已自动停止爬虫

stop-cancelled = 任务已取消

stop-other = 已结束

keyword-need-batch = 请先导入 CSV 并选择关键词批次

keyword-empty-batch = 批次 {$batch} 没有可用关键词
