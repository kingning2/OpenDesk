"""渠道工厂（参考 CowAgent channel/channel_factory.py）。"""

from __future__ import annotations

from channels.channel import Channel
from channels.core.platform_config import normalize_platform


def create_channel(channel_type: str | None) -> Channel:
    """按 Sidecar 传入的 channel_type 创建渠道实例。

    参数：
        channel_type: 请求体 ``platform`` 字段；空则默认闲鱼。

    异常：
        ValueError: 未知渠道。
    """
    name = normalize_platform(channel_type)
    ch: Channel

    if name == "xianyu":
        from channels.xianyu.xianyu_channel import XianyuChannel

        ch = XianyuChannel()
    elif name == "ali1688":
        from channels.ali1688.ali1688_channel import Ali1688Channel

        ch = Ali1688Channel()
    else:
        raise ValueError(f"不支持的渠道: {name}（支持: ali1688, xianyu）")

    ch.channel_type = name
    return ch
