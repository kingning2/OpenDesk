# crawler

Multi-platform crawl capability (YouTube mock first).

- Ports: `PlatformAdapter`, `CrawlEventEmitter`
- Process visibility: `crawler.job.log` events + `runtime/log/entry` (`feature=crawler`)
- No persistence, no outbound HTTP in skeleton (mock only)

## Sidecar routes

- `POST /v1/crawler/job/start`
- `POST /v1/crawler/job/cancel`
- `POST /v1/crawler/job/status`
