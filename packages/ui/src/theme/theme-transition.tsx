"use client";

import { useTheme } from "next-themes";
import * as React from "react";
import { flushSync } from "react-dom";

type Appearance = "light" | "dark";

function resolveAppearance(theme: string | undefined): Appearance {
  if (theme === "dark") return "dark";
  if (theme === "light") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function applyAppearance(appearance: Appearance) {
  const root = document.documentElement;
  root.classList.toggle("dark", appearance === "dark");
  root.style.colorScheme = appearance;
}

function startThemeViewTransition(update: () => void) {
  const start = document.startViewTransition?.bind(document);
  if (typeof start !== "function" || prefersReducedMotion()) {
    update();
    return;
  }

  try {
    start(update);
  } catch {
    update();
  }
}

type ThemeTransitionContextValue = {
  setThemeWithTransition: (theme: string) => void;
};

const ThemeTransitionContext = React.createContext<ThemeTransitionContextValue | null>(null);

export function ThemeTransitionProvider({ children }: { children: React.ReactNode }) {
  const { theme, setTheme } = useTheme();

  const setThemeWithTransition = React.useCallback(
    (next: string) => {
      if (typeof window === "undefined") {
        setTheme(next);
        return;
      }

      const currentAppearance = resolveAppearance(theme);
      const nextAppearance = resolveAppearance(next);

      if (currentAppearance === nextAppearance) {
        setTheme(next);
        return;
      }

      startThemeViewTransition(() => {
        applyAppearance(nextAppearance);
        flushSync(() => {
          setTheme(next);
        });
      });
    },
    [setTheme, theme],
  );

  return (
    <ThemeTransitionContext.Provider value={{ setThemeWithTransition }}>
      {children}
    </ThemeTransitionContext.Provider>
  );
}

export function useThemeTransition() {
  const context = React.useContext(ThemeTransitionContext);
  if (!context) {
    throw new Error("useThemeTransition must be used within ThemeProvider");
  }
  return context;
}
