/**
 * 桌面壳工作区标签条（路径式 tabs）。
 *
 * 高频操作：无进出场 / 共享布局动画；press 用 CSS `:active`。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { cn } from "@desk/ui";
import { Plus, X } from "@desk/ui/icons";
import { cva, type VariantProps } from "class-variance-authority";
import type { HTMLAttributes, ReactNode } from "react";

/**
 * 工作区标签项。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface TabBarItem {
  id: string;
  path: string;
  label: string;
  closable?: boolean;
}

/**
 * `TabBar` 属性。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface TabBarProps extends Omit<HTMLAttributes<HTMLDivElement>, "onSelect"> {
  /** 打开的标签列表。 */
  items: TabBarItem[];
  /** 当前激活路径。 */
  activePath: string;
  /** 选中标签。 */
  onSelect: (path: string) => void;
  /** 关闭标签。 */
  onClose?: (path: string) => void;
  /** 新增标签。 */
  onAdd?: () => void;
  /** 嵌入 TitleBar 的紧凑模式。 */
  embedded?: boolean;
}

/** Press 反馈：仅 transform，强 ease-out，100–160ms。 */
const pressableClass =
  "transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-[0.97]";

const tabItemVariants = cva(
  cn("group flex shrink-0 items-center gap-1.5 px-3.5 text-[length:var(--text-sm)]", pressableClass),
  {
    variants: {
      active: {
        true: "",
        false: "text-muted-foreground hover:bg-muted/30 hover:text-foreground",
      },
      embedded: {
        true: "h-8 rounded-[var(--radius-sm)]",
        false: "h-8 border-t-2 border-transparent",
      },
    },
    compoundVariants: [
      {
        active: true,
        embedded: false,
        class: "border-primary bg-workspace text-foreground",
      },
      {
        active: true,
        embedded: true,
        class: "text-foreground",
      },
    ],
    defaultVariants: {
      active: false,
      embedded: false,
    },
  },
);

interface TabItemProps extends VariantProps<typeof tabItemVariants> {
  item: TabBarItem;
  onSelect: (path: string) => void;
  onClose?: (path: string) => void;
  embedded?: boolean;
}

/**
 * 单个工作区标签。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 标签数据与回调
 * @returns 标签节点
 */
function TabItem({ item, active, embedded, onSelect, onClose }: TabItemProps) {
  const closable = item.closable ?? true;

  const content: ReactNode = (
    <>
      {active && embedded ? (
        <div
          className="absolute inset-0 rounded-[var(--radius-sm)] border border-border bg-workspace"
          aria-hidden
        />
      ) : null}
      <span className="relative z-10 max-w-32 truncate">{item.label}</span>
      {closable && onClose ? (
        <button
          type="button"
          aria-label={`Close ${item.label}`}
          onClick={(event) => {
            event.stopPropagation();
            onClose(item.path);
          }}
          className={cn(
            "relative z-10 inline-flex size-4 cursor-pointer items-center justify-center rounded-sm text-muted-foreground",
            pressableClass,
            "hover:bg-muted/60 hover:text-foreground",
            active
              ? "opacity-100"
              : "opacity-100 focus-visible:opacity-100 group-focus-within:opacity-100 [@media(hover:hover)_and_(pointer:fine)]:opacity-0 [@media(hover:hover)_and_(pointer:fine)]:group-hover:opacity-100",
          )}
        >
          <X className="size-3" />
        </button>
      ) : null}
    </>
  );

  return (
    <div
      className={cn(tabItemVariants({ active, embedded }), embedded && "relative", "cursor-pointer")}
      onClick={() => onSelect(item.path)}
    >
      {content}
    </div>
  );
}

/**
 * 工作区标签条。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 见 {@link TabBarProps}
 * @returns 标签条节点
 */
export function TabBar({
  items,
  activePath,
  onSelect,
  onClose,
  onAdd,
  embedded = false,
  className,
  ...props
}: TabBarProps) {
  return (
    <div
      className={cn(
        "flex min-w-0 items-stretch",
        embedded
          ? "shrink-0 self-stretch items-center gap-0.5 overflow-hidden px-1"
          : "h-9 shrink-0 border-b border-border bg-surface",
        className,
      )}
      {...props}
    >
      <div
        className={cn(
          "flex min-w-0 flex-1",
          embedded
            ? "items-center gap-0.5 overflow-hidden"
            : "items-stretch overflow-x-auto overflow-y-hidden [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
        )}
      >
        {items.map((item) => (
          <TabItem
            key={item.id}
            item={item}
            active={item.path === activePath}
            embedded={embedded}
            onSelect={onSelect}
            onClose={onClose}
          />
        ))}
      </div>
      {onAdd ? (
        <button
          type="button"
          aria-label="Open tab"
          onClick={onAdd}
          className={cn(
            "inline-flex w-9 shrink-0 cursor-pointer items-center justify-center text-muted-foreground",
            pressableClass,
            "hover:bg-muted/30 hover:text-foreground",
            embedded ? "h-8 w-8 rounded-[var(--radius-sm)]" : "border-l border-border",
          )}
        >
          <Plus className="size-3.5" />
        </button>
      ) : null}
    </div>
  );
}

export { tabItemVariants };
