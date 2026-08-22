import { cn } from "@desk/ui";

/** 圆形头像：有 URL 显示图片，否则显示名称首字。 */
export function ConversationAvatar({
  name,
  src,
  size = "md",
}: {
  name: string;
  src?: string | null;
  size?: "sm" | "md";
}) {
  const dim = size === "sm" ? "size-7" : "size-9";
  const text = size === "sm" ? "text-[length:var(--text-xs)]" : "text-[length:var(--text-sm)]";
  const initial = (name.trim() || "?").slice(0, 1);
  if (src) {
    return (
      <img
        src={src}
        alt=""
        className={cn(dim, "shrink-0 rounded-full border border-border object-cover")}
      />
    );
  }
  return (
    <div
      className={cn(
        dim,
        text,
        "flex shrink-0 items-center justify-center rounded-full border border-border bg-muted font-medium text-muted-foreground",
      )}
      aria-hidden
    >
      {initial}
    </div>
  );
}
