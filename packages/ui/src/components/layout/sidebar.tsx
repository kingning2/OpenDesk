/**
 * Aceternity 可折叠侧栏 — 侧栏宽度折叠 + 菜单分组 accordion。
 *
 * @see https://ui.aceternity.com/components/sidebar
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ComponentProps,
  type Dispatch,
  type ReactNode,
  type SetStateAction,
} from "react";
import { motion } from "motion/react";
import { ChevronDown, ChevronLeft, ChevronRight } from "lucide-react";

import { cn } from "../../lib/cn";
import { ScrollArea } from "../scroll-area";

/** 收起宽度（px）。 */
export const SIDEBAR_WIDTH_COLLAPSED = 60;
/** 展开宽度（px）。 */
export const SIDEBAR_WIDTH_EXPANDED = 240;

/**
 * 侧栏上下文。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarContextValue {
  expanded: boolean;
  setExpanded: Dispatch<SetStateAction<boolean>>;
  toggleExpanded: () => void;
  /** 与 expanded 相同（侧栏宽度是否展开）。 */
  open: boolean;
  animate: boolean;
}

const SidebarContext = createContext<SidebarContextValue | undefined>(undefined);

/**
 * 读取侧栏上下文。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function useSidebar(): SidebarContextValue {
  const context = useContext(SidebarContext);
  if (!context) {
    throw new Error("useSidebar must be used within SidebarProvider");
  }
  return context;
}

/**
 * 菜单分组折叠上下文。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarGroupsContextValue {
  isGroupOpen: (groupId: string) => boolean;
  toggleGroup: (groupId: string) => void;
}

const SidebarGroupsContext = createContext<SidebarGroupsContextValue | undefined>(undefined);

/**
 * 读取菜单分组折叠状态。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function useSidebarGroups(): SidebarGroupsContextValue {
  const context = useContext(SidebarGroupsContext);
  if (!context) {
    throw new Error("useSidebarGroups must be used within SidebarGroupsProvider");
  }
  return context;
}

/**
 * 侧栏 Provider 属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarProviderProps {
  children: ReactNode;
  open?: boolean;
  setOpen?: Dispatch<SetStateAction<boolean>>;
  defaultOpen?: boolean;
  animate?: boolean;
}

/**
 * 侧栏状态 Provider。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function SidebarProvider({
  children,
  open: openProp,
  setOpen: setOpenProp,
  defaultOpen = true,
  animate = true,
}: SidebarProviderProps) {
  const [expandedState, setExpandedState] = useState(defaultOpen);

  const expanded = openProp ?? expandedState;
  const setExpanded = setOpenProp ?? setExpandedState;

  const toggleExpanded = useCallback(() => {
    setExpanded((value) => !value);
  }, [setExpanded]);

  const value = useMemo(
    () => ({
      expanded,
      setExpanded,
      toggleExpanded,
      open: expanded,
      animate,
    }),
    [expanded, setExpanded, toggleExpanded, animate],
  );

  return <SidebarContext.Provider value={value}>{children}</SidebarContext.Provider>;
}

/**
 * 菜单分组折叠 Provider 属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarGroupsProviderProps {
  children: ReactNode;
  /** localStorage 键；不传则不持久化。 */
  storageKey?: string;
  /** 初始展开的分组 id。 */
  defaultOpenGroupIds?: string[];
  /** 含当前路由时分组自动展开（不强制收起其它组）。 */
  autoOpenGroupIds?: string[];
}

function readGroupState(storageKey: string): Record<string, boolean> {
  if (typeof window === "undefined") {
    return {};
  }
  try {
    const raw = window.localStorage.getItem(storageKey);
    if (!raw) {
      return {};
    }
    const parsed: unknown = JSON.parse(raw);
    return typeof parsed === "object" && parsed !== null ? (parsed as Record<string, boolean>) : {};
  } catch {
    return {};
  }
}

/**
 * 管理侧栏菜单分组的展开/收起。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function SidebarGroupsProvider({
  children,
  storageKey,
  defaultOpenGroupIds = [],
  autoOpenGroupIds = [],
}: SidebarGroupsProviderProps) {
  const [groupState, setGroupState] = useState<Record<string, boolean>>(() => {
    const persisted = storageKey ? readGroupState(storageKey) : {};
    const initial: Record<string, boolean> = { ...persisted };
    for (const id of defaultOpenGroupIds) {
      if (!(id in initial)) {
        initial[id] = true;
      }
    }
    return initial;
  });

  useEffect(() => {
    if (autoOpenGroupIds.length === 0) {
      return;
    }
    setGroupState((prev) => {
      let changed = false;
      const next = { ...prev };
      for (const id of autoOpenGroupIds) {
        if (!next[id]) {
          next[id] = true;
          changed = true;
        }
      }
      return changed ? next : prev;
    });
  }, [autoOpenGroupIds]);

  useEffect(() => {
    if (!storageKey) {
      return;
    }
    window.localStorage.setItem(storageKey, JSON.stringify(groupState));
  }, [groupState, storageKey]);

  const isGroupOpen = useCallback(
    (groupId: string) => groupState[groupId] ?? false,
    [groupState],
  );

  const toggleGroup = useCallback((groupId: string) => {
    setGroupState((prev) => ({
      ...prev,
      [groupId]: !(prev[groupId] ?? false),
    }));
  }, []);

  const value = useMemo(
    () => ({ isGroupOpen, toggleGroup }),
    [isGroupOpen, toggleGroup],
  );

  return <SidebarGroupsContext.Provider value={value}>{children}</SidebarGroupsContext.Provider>;
}

/**
 * 桌面侧栏容器。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function DesktopSidebar({
  className,
  children,
  ...props
}: ComponentProps<typeof motion.aside>) {
  const { open, animate } = useSidebar();

  return (
    <motion.aside
      className={cn(
        "hidden h-full shrink-0 flex-col border-r border-border bg-shell py-3 md:flex",
        className,
      )}
      animate={{
        width: animate ? (open ? SIDEBAR_WIDTH_EXPANDED : SIDEBAR_WIDTH_COLLAPSED) : SIDEBAR_WIDTH_EXPANDED,
      }}
      transition={{ duration: 0.22, ease: [0.23, 1, 0.32, 1] }}
      {...props}
    >
      {children}
    </motion.aside>
  );
}

/**
 * 侧栏宽度折叠按钮属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarToggleProps {
  className?: string;
  /** `footer`：固定在侧栏底部，展开时占满宽度。 */
  placement?: "inline" | "footer";
}

/**
 * 侧栏宽度折叠按钮。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function SidebarToggle({ className, placement = "inline" }: SidebarToggleProps) {
  const { expanded, toggleExpanded, open, animate } = useSidebar();
  const label = expanded ? "收起侧栏" : "展开侧栏";

  return (
    <button
      type="button"
      onClick={toggleExpanded}
      aria-label={label}
      aria-expanded={expanded}
      title={label}
      className={cn(
        "flex shrink-0 cursor-pointer items-center rounded-[var(--radius-md)] text-muted-foreground transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] hover:bg-muted/50 hover:text-foreground active:scale-[0.97]",
        placement === "inline" && "size-8 justify-center",
        placement === "footer" && "h-9 w-full justify-center px-2.5",
        placement === "footer" && expanded && "justify-end",
        className,
      )}
    >
      {placement === "footer" ? (
        <>
          <motion.span
            animate={{
              display: animate ? (open ? "inline-block" : "none") : "inline-block",
              opacity: animate ? (open ? 1 : 0) : 1,
            }}
            className="mr-auto truncate text-[length:var(--text-sm)]"
          >
            {label}
          </motion.span>
          {expanded ? (
            <ChevronLeft className="size-4 shrink-0" aria-hidden />
          ) : (
            <ChevronRight className="size-4 shrink-0" aria-hidden />
          )}
        </>
      ) : expanded ? (
        <ChevronLeft className="size-4" aria-hidden />
      ) : (
        <ChevronRight className="size-4" aria-hidden />
      )}
    </button>
  );
}

/**
 * 侧栏链接属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarLinkProps extends ComponentProps<"button"> {
  label: string;
  icon?: ReactNode;
  active?: boolean;
}

/**
 * 侧栏导航项。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function SidebarLink({
  label,
  icon,
  active = false,
  className,
  ...props
}: SidebarLinkProps) {
  const { open, animate } = useSidebar();

  return (
    <button
      type="button"
      title={label}
      className={cn(
        "group/sidebar flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-md)] px-2.5 py-2 text-left text-[length:var(--text-sm)] transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-[0.98]",
        active
          ? "bg-primary/15 text-primary"
          : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
        !open && "justify-center px-2",
        className,
      )}
      {...props}
    >
      {icon ? <span className="flex size-5 shrink-0 items-center justify-center">{icon}</span> : null}
      <motion.span
        animate={{
          display: animate ? (open ? "inline-block" : "none") : "inline-block",
          opacity: animate ? (open ? 1 : 0) : 1,
        }}
        className="truncate whitespace-nowrap"
      >
        {label}
      </motion.span>
    </button>
  );
}

/**
 * 侧栏分组属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SidebarGroupProps {
  /** 分组唯一 id（用于持久化折叠状态）。 */
  groupId: string;
  label: string;
  /** 分组图标（收起侧栏时仅显示图标）。 */
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
}

/**
 * 可折叠菜单分组 — 点击组标题展开/收起子项。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function SidebarGroup({ groupId, label, icon, children, className }: SidebarGroupProps) {
  const { open: sidebarOpen, animate } = useSidebar();
  const { isGroupOpen, toggleGroup } = useSidebarGroups();
  const groupOpen = isGroupOpen(groupId);

  if (!sidebarOpen) {
    return (
      <div className={cn("space-y-0.5", className)}>
        <button
          type="button"
          title={label}
          aria-expanded={groupOpen}
          onClick={() => toggleGroup(groupId)}
          className={cn(
            "flex w-full cursor-pointer items-center justify-center rounded-[var(--radius-md)] p-2 transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-[0.98]",
            groupOpen
              ? "bg-primary/15 text-primary"
              : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
          )}
        >
          {icon ? <span className="flex size-5 shrink-0 items-center justify-center">{icon}</span> : null}
        </button>
        {groupOpen ? <div className="space-y-0.5">{children}</div> : null}
      </div>
    );
  }

  return (
    <div className={cn("space-y-0.5", className)}>
      <button
        type="button"
        onClick={() => toggleGroup(groupId)}
        aria-expanded={groupOpen}
        className="flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-md)] px-2.5 pt-3 pb-1.5 text-left text-[length:var(--text-sm)] font-medium text-muted-foreground transition-colors hover:text-foreground"
      >
        {icon ? <span className="flex size-5 shrink-0 items-center justify-center">{icon}</span> : null}
        <span className="min-w-0 flex-1 truncate">{label}</span>
        <ChevronDown
          className={cn(
            "size-3.5 shrink-0 transition-transform duration-200 ease-[cubic-bezier(0.23,1,0.32,1)]",
            !groupOpen && "-rotate-90",
          )}
          aria-hidden
        />
      </button>
      <div
        className={cn(
          "grid transition-[grid-template-rows] duration-200 ease-[cubic-bezier(0.23,1,0.32,1)]",
          animate && (groupOpen ? "grid-rows-[1fr]" : "grid-rows-[0fr]"),
          !animate && (groupOpen ? "grid-rows-[1fr]" : "hidden"),
        )}
      >
        <div className="overflow-hidden">
          <div className="space-y-0.5 pb-1 pl-1">{children}</div>
        </div>
      </div>
    </div>
  );
}

/**
 * 侧栏可滚动内容区。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function SidebarContent({ className, children }: { className?: string; children: ReactNode }) {
  return (
    <ScrollArea className={cn("min-h-0 flex-1", className)}>
      <div className="space-y-0.5 px-2">{children}</div>
    </ScrollArea>
  );
}

export function SidebarFooter({ className, children }: { className?: string; children: ReactNode }) {
  return <div className={cn("shrink-0 px-2 pt-2", className)}>{children}</div>;
}

export function SidebarHeader({ className, children }: { className?: string; children: ReactNode }) {
  return <div className={cn("shrink-0 space-y-1 px-2 pb-2", className)}>{children}</div>;
}
