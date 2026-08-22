"""Auto-generated from contracts/schema."""

from typing import TypedDict
from .ai_account import AiAccount
from .ai_provider import AiProvider


class AiIpcConfigRequest(TypedDict):
    providers: list[AiProvider]
    accounts: list[AiAccount]
