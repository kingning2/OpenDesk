"""Auto-generated from contracts/schema."""

from typing import TypedDict


class WorkflowIpcSnippetSaveRequest(TypedDict, total=False):
    id: str
    title: str
    stage: str
    trigger_text: str
    description: str
    from_stage: str
    to_stage: str
    tags_json: str
    body_text: str
    category_l1: str
    category_l2: str
    needs_boss_input: bool
    boss_input_hint: str
    sort_order: int
