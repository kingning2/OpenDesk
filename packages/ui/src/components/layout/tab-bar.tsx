import { Plus, X } from "../../icons";
import { cva, type VariantProps } from "class-variance-authority";
import { AnimatePresence, LayoutGroup, motion } from "motion/react";
import * as React from "react";

import { cn } from "../../lib/cn";
import { spring } from "../../tokens/motion";

export interface TabBarItem {
  id: string;
  path: string;
  label: string;
  closable?: boolean;
}

export interface TabBarProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "onSelect"> {
  items: TabBarItem[];
  activePath: string;
  onSelect: (path: string) => void;
  onClose?: (path: string) => void;
  onAdd?: () => void;
  embedded?: boolean;
}

const tabItemVariants = cva(
  "group flex shrink-0 items-center gap-1.5 px-3.5 text-[length:var(--text-sm)] transition-colors",
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

function TabItem({ item, active, embedded, onSelect, onClose }: TabItemProps) {
  const closable = item.closable ?? true;

  const content = (
    <>
      {active && embedded ? (
        <motion.div
          layoutId="workspace-tab-active"
          className="absolute inset-0 rounded-[var(--radius-sm)] border border-border bg-workspace"
          transition={spring.snappy}
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
            "relative z-10 inline-flex size-4 cursor-pointer items-center justify-center rounded-sm text-muted-foreground transition-opacity hover:bg-muted/60 hover:text-foreground",
            active ? "opacity-100" : "opacity-0 group-hover:opacity-100",
          )}
        >
          <X className="size-3" />
        </button>
      ) : null}
    </>
  );

  if (embedded) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={spring.snappy}
        className={cn(tabItemVariants({ active, embedded }), "relative cursor-pointer")}
        onClick={() => onSelect(item.path)}
      >
        {content}
      </motion.div>
    );
  }

  return (
    <div
      className={cn(tabItemVariants({ active, embedded }), "cursor-pointer")}
      onClick={() => onSelect(item.path)}
    >
      {content}
    </div>
  );
}

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
      <LayoutGroup id="workspace-tabs">
        <div
          className={cn(
            "flex min-w-0 flex-1",
            embedded
              ? "items-center gap-0.5 overflow-hidden"
              : "items-stretch overflow-x-auto overflow-y-hidden [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
          )}
        >
          <AnimatePresence initial={false}>
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
          </AnimatePresence>
        </div>
      </LayoutGroup>
      {onAdd ? (
        <motion.button
          type="button"
          aria-label="Open tab"
          onClick={onAdd}
          whileTap={{ scale: 0.94 }}
          transition={spring.snappy}
          className={cn(
            "inline-flex w-9 shrink-0 cursor-pointer items-center justify-center text-muted-foreground transition-colors hover:bg-muted/30 hover:text-foreground",
            embedded
              ? "h-8 w-8 rounded-[var(--radius-sm)]"
              : "border-l border-border",
          )}
        >
          <Plus className="size-3.5" />
        </motion.button>
      ) : null}
    </div>
  );
}

export { tabItemVariants };
