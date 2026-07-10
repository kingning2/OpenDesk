/** Motion spring presets — Apple-like snappy / smooth interactions. */

export const spring = {
  /** UI controls, toggles */
  snappy: { type: "spring" as const, stiffness: 520, damping: 34, mass: 0.7 },
  /** Panels, sheets */
  smooth: { type: "spring" as const, stiffness: 320, damping: 32, mass: 0.9 },
  /** Large layout shifts */
  gentle: { type: "spring" as const, stiffness: 220, damping: 28, mass: 1.1 },
} as const;

export const duration = {
  fast: "var(--motion-fast)",
  base: "var(--motion-base)",
  slow: "var(--motion-slow)",
} as const;
