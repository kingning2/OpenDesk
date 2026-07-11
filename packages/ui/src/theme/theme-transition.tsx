"use client";

import { useTheme } from "next-themes";
import * as React from "react";
import { createPortal } from "react-dom";

import { DiagonalReveal } from "../motion/diagonal-reveal";
import { spring } from "../tokens/motion";

type Appearance = "light" | "dark";

function resolveAppearance(theme: string | undefined): Appearance {
  if (theme === "dark") return "dark";
  if (theme === "light") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

type ThemeTransitionContextValue = {
  setThemeWithTransition: (theme: string) => void;
};

const ThemeTransitionContext = React.createContext<ThemeTransitionContextValue | null>(null);

function ThemeTransitionOverlay({
  appearance,
  onComplete,
}: {
  appearance: Appearance;
  onComplete: () => void;
}) {
  return createPortal(
    <div
      className={appearance === "dark" ? "dark" : undefined}
      aria-hidden
      onPointerDown={(event) => event.preventDefault()}
    >
      <DiagonalReveal
        transition={spring.smooth}
        className="fixed inset-0 z-[9999] bg-shell"
        onAnimationComplete={onComplete}
      />
    </div>,
    document.body,
  );
}

export function ThemeTransitionProvider({ children }: { children: React.ReactNode }) {
  const { theme, setTheme } = useTheme();
  const [overlayAppearance, setOverlayAppearance] = React.useState<Appearance | null>(null);
  const pendingTheme = React.useRef<string | null>(null);

  const setThemeWithTransition = React.useCallback(
    (next: string) => {
      if (typeof window === "undefined") {
        setTheme(next);
        return;
      }

      if (prefersReducedMotion()) {
        setTheme(next);
        return;
      }

      const currentAppearance = resolveAppearance(theme);
      const nextAppearance = resolveAppearance(next);

      if (currentAppearance === nextAppearance) {
        setTheme(next);
        return;
      }

      pendingTheme.current = next;
      setOverlayAppearance(nextAppearance);
    },
    [setTheme, theme],
  );

  const completeTransition = React.useCallback(() => {
    if (pendingTheme.current) {
      setTheme(pendingTheme.current);
      pendingTheme.current = null;
    }
    setOverlayAppearance(null);
  }, [setTheme]);

  return (
    <ThemeTransitionContext.Provider value={{ setThemeWithTransition }}>
      {children}
      {overlayAppearance ? (
        <ThemeTransitionOverlay appearance={overlayAppearance} onComplete={completeTransition} />
      ) : null}
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
