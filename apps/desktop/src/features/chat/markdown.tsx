/**
 * AI 消息 Markdown 渲染：react-markdown + remark-gfm。
 *
 * 流式输出时内容经常是不完整的（"一半标签"）：未闭合的代码围栏、悬空的 ** 加粗、
 * 写了一半的表格。react-markdown 按 CommonMark 对不完整输入天然宽容（悬空标记渲染为
 * 字面文本、未闭合围栏延伸到结尾），唯一可能破坏布局的是未闭合代码围栏——它会吞掉后续
 * 内容。`closeUnclosedFence` 在解析前把处于未闭合围栏中的输入补一个结尾围栏，让流式
 * 期间的代码块稳定渲染，不会把后面的正文吞进去。
 *
 * @author coisini
 */

import { memo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import "./markdown.css";

/**
 * 逐行跟踪围栏状态：若输入结束时仍处于未闭合代码围栏内，追加一个相同字符的结尾围栏。
 * 仅在检测到奇数个围栏线时生效，正常内容不受影响。
 */
function closeUnclosedFence(text: string): string {
  let openMarker: string | null = null;
  for (const line of text.split("\n")) {
    const match = /^ {0,3}(`{3,}|~{3,})[^`~]*$/.exec(line);
    if (!match) {
      continue;
    }
    const marker = match[1];
    if (openMarker === null) {
      openMarker = marker;
    } else if (marker[0] === openMarker[0] && marker.length >= openMarker.length) {
      openMarker = null;
    }
  }
  if (openMarker !== null) {
    return `${text}\n${openMarker}`;
  }
  return text;
}

/**
 * 渲染一条 AI 消息正文为 Markdown。
 *
 * @author coisini
 *
 * @param props.content - Markdown 源文本（可能不完整）
 * @param props.className - 附加到根节点的类名
 * @returns 渲染后的正文节点
 */
export const MarkdownContent = memo(function MarkdownContent({
  content,
  className,
}: {
  content: string;
  className?: string;
}) {
  return (
    <div className={className ? `markdown-body ${className}` : "markdown-body"}>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>
        {closeUnclosedFence(content)}
      </ReactMarkdown>
    </div>
  );
});
