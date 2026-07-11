"use client";

import { Monitor, Moon, Sun } from "../icons";
import { motion } from "motion/react";
import { useTheme } from "next-themes";
import * as React from "react";

import { cn } from "../lib/cn";
import { spring } from "../tokens/motion";
import { useThemeTransition } from "../theme/theme-transition";

const options = [
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
  { value: "system", label: "System", icon: Monitor },
] as const;

export type ThemeToggleProps = React.HTMLAttributes<HTMLDivElement> & {
  size?: "default" | "compact";
};

function useIsMounted() {
  return React.useSyncExternalStore(
    () => () => {},
    () => true,
    () => false,
  );
}

export function ThemeToggle({ className, size = "default", ...props }: ThemeToggleProps) {
  const { theme } = useTheme();
  const { setThemeWithTransition } = useThemeTransition();
  const mounted = useIsMounted();
  const compact = size === "compact";

  if (!mounted) {
    return (
      <div
        aria-hidden
        className={cn(
          compact ? "h-8 w-[5.25rem] rounded-[var(--radius-sm)] bg-muted/30" : "h-8 w-[7.5rem] rounded-[var(--radius-md)] bg-muted/40",
          className,
        )}
      />
    );
  }

  return (
    <div
      role="group"
      aria-label="Theme"
      className={cn(
        "inline-flex rounded-[var(--radius-sm)] p-0.5",
        compact ? "bg-transparent" : "rounded-[var(--radius-md)] border border-border bg-muted/40",
        className,
      )}
      {...props}
    >
      {options.map(({ value, label, icon: Icon }) => {
        const active = theme === value;
        return (
          <button
            key={value}
            type="button"
            aria-label={label}
            aria-pressed={active}
            onClick={() => setThemeWithTransition(value)}
            className={cn(
              "relative inline-flex cursor-pointer items-center justify-center rounded-[calc(var(--radius-sm)-1px)] transition-colors",
              compact ? "size-7" : "rounded-[calc(var(--radius-md)-2px)] px-2 py-1",
              active ? "text-foreground" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {active ? (
              <motion.span
                layoutId="theme-toggle-active"
                className={cn(
                  "absolute inset-0 rounded-[calc(var(--radius-sm)-1px)] bg-muted/60",
                  !compact && "rounded-[calc(var(--radius-md)-2px)]",
                )}
                transition={spring.snappy}
              />
            ) : null}
            <Icon className={cn("relative z-10", compact ? "size-3" : "size-3.5")} />
          </button>
        );
      })}
    </div>
  );
}
