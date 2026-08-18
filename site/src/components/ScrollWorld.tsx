'use client';

import { useEffect, useRef } from 'react';
import { mountScrollWorld } from '@/lib/scroll-world';
import { SECTIONS } from '@/lib/sections';

/**
 * scroll-world 引擎封装。
 * 素材后补：`connectors` 现为空数组，dive 之间直接交叉淡化；
 * 生成场景视频后填入 `public/assets/vid/` 并在此引用（长度 = SECTIONS - 1）。
 */
export default function ScrollWorld() {
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    mountScrollWorld(host, {
      brand: { name: 'OpenDesk', href: '#top' },
      cta: { label: '下载', href: 'https://github.com/kingning2/OpenDesk/releases' },
      hint: '向下滚动 · 飞进卖家的世界',
      diveScroll: 1.3,
      connScroll: 0.9,
      sections: SECTIONS,
      connectors: [],
      connectorsMobile: [],
    });
    // 引擎自建 DOM 与事件监听；整站为静态单页，无需卸载逻辑。
  }, []);

  return <div id="world" ref={hostRef} />;
}
