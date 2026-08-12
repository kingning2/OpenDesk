"""Sidecar handler: /v1/llm/classify (POST) — Python ← Rust only.

纯大模型接入：给定文本与候选意图，让模型返回其一。不含业务/渠道逻辑。
"""

from __future__ import annotations

import logging
from typing import Any

from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.llm")

try:  # openai 为可选运行时依赖；缺失时返回可读错误。
    from openai import OpenAI
except ImportError:  # pragma: no cover
    OpenAI = None  # type: ignore[assignment]


def handle_llm_classify(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/llm/ipc/classify.request/response.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="llm"):
        if payload is None:
            return {"intent": "", "trace_id": trace_id}
        text = payload.get("text") or ""
        options = payload.get("options") or []
        provider = payload.get("provider") or {}

        if OpenAI is None:
            logger.warning("openai 未安装，无法分类", extra={"event": "llm.classify.missing_dep"})
            return {"intent": options[0] if options else "", "trace_id": trace_id}

        base_url = provider.get("base_url") or ""
        api_key = provider.get("api_key") or ""
        model = provider.get("model") or ""

        if not base_url or not api_key or not model:
            return {"intent": options[0] if options else "", "trace_id": trace_id}

        option_list = "、".join(options) if options else ""
        prompt = (
            f"以下是买家在闲鱼上发来的一句话。请判断其意图，只能从以下选项中选一个："
            f"[{option_list}]。直接返回选项原文，不要任何解释或标点。\n\n买家消息：{text}"
        )

        try:
            client = OpenAI(base_url=base_url, api_key=api_key)
            completion = client.chat.completions.create(
                model=model,
                messages=[
                    {"role": "system", "content": "你是意图分类器，只输出给定选项中的原文。"},
                    {"role": "user", "content": prompt},
                ],
            )
            raw = (completion.choices[0].message.content or "").strip()
            intent = next((o for o in options if o in raw), options[0] if options else "")
            logger.info("LLM 意图分类成功", extra={"event": "llm.classify.ok", "intent": intent})
            return {"intent": intent, "trace_id": trace_id}
        except Exception:  # noqa: BLE001
            logger.exception("LLM 意图分类失败", extra={"event": "llm.classify.failed"})
            return {"intent": options[0] if options else "", "trace_id": trace_id}
