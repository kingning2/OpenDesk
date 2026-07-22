/**
 * Sandboxed HTML email preview (DOMPurify + iframe).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { memo, useEffect, useMemo, useRef } from "react";
import { cn } from "@desk/ui";

import { resolveMailPreviewHtml, sanitizeMailHtml, wrapMailHtmlDocument } from "./mail-html";

/**
 * Render one mail message body as real HTML inside a sandboxed iframe.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export const MailHtmlPreview = memo(function MailHtmlPreview({
  bodyHtml,
  bodyText,
  emptyLabel,
  className,
  minHeight = 180,
}: {
  bodyHtml?: string;
  bodyText?: string;
  emptyLabel: string;
  className?: string;
  minHeight?: number;
}) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const srcDoc = useMemo(() => {
    const raw = resolveMailPreviewHtml(bodyHtml, bodyText);
    if (!raw) {
      return "";
    }
    return wrapMailHtmlDocument(sanitizeMailHtml(raw));
  }, [bodyHtml, bodyText]);

  useEffect(() => {
    const iframe = iframeRef.current;
    if (!iframe || !srcDoc) {
      return;
    }
    const resize = () => {
      try {
        const doc = iframe.contentDocument;
        const height = doc?.documentElement?.scrollHeight ?? minHeight;
        iframe.style.height = `${Math.max(minHeight, height)}px`;
      } catch {
        iframe.style.height = `${minHeight}px`;
      }
    };
    iframe.addEventListener("load", resize);
    resize();
    return () => iframe.removeEventListener("load", resize);
  }, [srcDoc, minHeight]);

  if (!srcDoc) {
    return (
      <p className={cn("text-sm text-muted-foreground", className)}>{emptyLabel}</p>
    );
  }

  return (
    <iframe
      ref={iframeRef}
      title="mail-html-preview"
      className={cn("w-full rounded-[var(--radius-md)] border border-border/60 bg-white", className)}
      style={{ minHeight }}
      sandbox=""
      srcDoc={srcDoc}
    />
  );
});
