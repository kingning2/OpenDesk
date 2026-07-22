/**
 * HTML mail sanitization and plain-text fallbacks.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import DOMPurify from "dompurify";

/** Tags commonly used in business email HTML. */
const EMAIL_HTML_ALLOWED_TAGS = [
  "a",
  "b",
  "blockquote",
  "br",
  "div",
  "em",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "hr",
  "i",
  "img",
  "li",
  "ol",
  "p",
  "pre",
  "span",
  "strong",
  "table",
  "tbody",
  "td",
  "th",
  "thead",
  "tr",
  "u",
  "ul",
];

/**
 * Sanitize untrusted email HTML for in-app preview.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param html - Raw HTML from mail storage or compose
 * @returns Sanitized HTML safe for iframe `srcDoc`
 */
export function sanitizeMailHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: EMAIL_HTML_ALLOWED_TAGS,
    ALLOWED_ATTR: [
      "href",
      "src",
      "alt",
      "title",
      "width",
      "height",
      "style",
      "class",
      "target",
      "rel",
      "colspan",
      "rowspan",
      "align",
      "valign",
      "border",
      "cellpadding",
      "cellspacing",
    ],
    ALLOW_DATA_ATTR: false,
  });
}

/**
 * Wrap fragment HTML in a minimal document for iframe rendering.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param html - Sanitized HTML fragment
 * @returns Full HTML document string
 */
export function wrapMailHtmlDocument(html: string): string {
  return `<!DOCTYPE html><html><head><meta charset="utf-8"><base target="_blank" rel="noopener noreferrer"><style>
body{margin:0;padding:12px 14px;font-family:system-ui,-apple-system,sans-serif;font-size:14px;line-height:1.65;color:#1f2937;word-break:break-word;overflow-wrap:anywhere;background:#fff}
img{max-width:100%;height:auto}
a{color:#2563eb}
table{max-width:100%}
html{scrollbar-width:thin;scrollbar-color:rgba(31,41,55,.32) transparent}
::-webkit-scrollbar{width:8px;height:8px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background-color:rgba(31,41,55,.32);border:2px solid transparent;border-radius:9999px;background-clip:padding-box}
::-webkit-scrollbar-thumb:hover{background-color:rgba(31,41,55,.48)}
</style></head><body>${html}</body></html>`;
}

/**
 * Strip tags to plain text (compose fallback / list search).
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param html - HTML string
 * @returns Plain text
 */
export function stripMailHtml(html: string | undefined): string {
  if (!html) {
    return "";
  }
  return html
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<\/p>/gi, "\n")
    .replace(/<\/div>/gi, "\n")
    .replace(/<[^>]+>/g, "")
    .replace(/&nbsp;/g, " ")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&")
    .trim();
}

/**
 * Pick preview HTML or synthesize from plain text.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param bodyHtml - Stored HTML body
 * @param bodyText - Plain text fallback
 * @returns HTML suitable for preview, or empty string
 */
export function resolveMailPreviewHtml(bodyHtml: string | undefined, bodyText: string | undefined): string {
  const trimmedHtml = bodyHtml?.trim();
  if (trimmedHtml) {
    return trimmedHtml;
  }
  const text = bodyText?.trim();
  if (!text) {
    return "";
  }
  const escaped = text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
  return `<div style="white-space:pre-wrap">${escaped}</div>`;
}
