/**
 * SafeHtml — XSS 安全 HTML 渲染。
 *
 * 白名单标签 + 属性过滤 + 协议校验；从原前端 `components/common/SafeHtml.tsx`
 * 抽取为公共组件（公告/素材详情等富文本展示复用）。
 */

import { useMemo } from "react";

import { cn } from "../lib/cn";

const ALLOWED_TAGS = new Set(["a", "b", "br", "div", "em", "i", "p", "small", "span", "strong", "u"]);
const STRIP_CONTENT_TAGS = new Set([
  "button",
  "embed",
  "form",
  "iframe",
  "input",
  "object",
  "script",
  "select",
  "style",
  "textarea",
]);
const ALLOWED_PROTOCOLS = new Set(["http:", "https:", "mailto:", "tel:"]);
const ALLOWED_ATTRIBUTES: Record<string, Set<string>> = {
  a: new Set(["href", "target", "title"]),
};

function sanitizeHref(href: string): string | null {
  const trimmed = href.trim();
  if (!trimmed) {
    return null;
  }
  if (trimmed.startsWith("/") || trimmed.startsWith("#")) {
    return trimmed;
  }
  try {
    const parsed = new URL(trimmed, window.location.origin);
    if (ALLOWED_PROTOCOLS.has(parsed.protocol)) {
      return trimmed;
    }
  } catch {
    return null;
  }
  return null;
}

function sanitizeNode(node: ChildNode): void {
  if (node.nodeType === Node.TEXT_NODE) {
    return;
  }
  if (node.nodeType !== Node.ELEMENT_NODE) {
    node.remove();
    return;
  }

  const element = node as HTMLElement;
  const tagName = element.tagName.toLowerCase();

  if (!ALLOWED_TAGS.has(tagName)) {
    if (STRIP_CONTENT_TAGS.has(tagName)) {
      element.remove();
      return;
    }
    const parent = element.parentNode;
    if (!parent) {
      element.remove();
      return;
    }
    const childNodes = Array.from(element.childNodes);
    for (const childNode of childNodes) {
      parent.insertBefore(childNode, element);
      sanitizeNode(childNode as ChildNode);
    }
    element.remove();
    return;
  }

  const allowedAttributes = ALLOWED_ATTRIBUTES[tagName] ?? new Set<string>();
  for (const attribute of Array.from(element.attributes)) {
    const attributeName = attribute.name.toLowerCase();
    if (attributeName.startsWith("on") || !allowedAttributes.has(attributeName)) {
      element.removeAttribute(attribute.name);
      continue;
    }
    if (tagName === "a" && attributeName === "href") {
      const safeHref = sanitizeHref(attribute.value);
      if (safeHref) {
        element.setAttribute("href", safeHref);
      } else {
        element.removeAttribute(attribute.name);
      }
    }
  }

  if (tagName === "a" && element.getAttribute("target") === "_blank") {
    element.setAttribute("rel", "noopener noreferrer");
  }

  for (const childNode of Array.from(element.childNodes)) {
    sanitizeNode(childNode as ChildNode);
  }
}

function sanitizeHtml(html: string): string {
  if (!html.trim() || typeof document === "undefined") {
    return html;
  }
  const template = document.createElement("template");
  template.innerHTML = html;
  for (const childNode of Array.from(template.content.childNodes)) {
    sanitizeNode(childNode as ChildNode);
  }
  return template.innerHTML;
}

export interface SafeHtmlProps {
  html: string;
  className?: string;
}

/**
 * 安全 HTML 渲染。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @param props - 见 {@link SafeHtmlProps}
 * @returns 富文本节点
 */
export function SafeHtml({ html, className }: SafeHtmlProps) {
  const sanitized = useMemo(() => sanitizeHtml(html), [html]);

  return (
    <div
      className={cn(
        "break-words leading-6 [&_a]:text-blue-600 [&_a]:underline-offset-2 [&_a]:hover:underline dark:[&_a]:text-blue-400",
        className,
      )}
      dangerouslySetInnerHTML={{ __html: sanitized }}
    />
  );
}
