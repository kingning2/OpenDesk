/**
 * Centered loading placeholder with spinner animation.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import type { ReactNode } from "react";

import { cn } from "../lib/cn";
import { Spinner, type SpinnerProps } from "./spinner";

export interface LoadingStateProps {
  /** Optional status label under the spinner */
  label?: ReactNode;
  /** Spinner size */
  size?: SpinnerProps["size"];
  /** Extra class names for the container */
  className?: string;
}

/**
 * Panel / list loading surface with rotating spinner.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param props.label - Optional text shown under the spinner
 * @param props.size - Spinner size
 * @param props.className - Container class override
 * @returns Accessible loading status region
 */
export function LoadingState({ label, size = "md", className }: LoadingStateProps) {
  return (
    <div
      role="status"
      aria-busy="true"
      aria-live="polite"
      className={cn(
        "flex min-h-32 flex-1 flex-col items-center justify-center gap-2 px-4 py-10 text-muted-foreground",
        className,
      )}
    >
      <Spinner size={size} className="text-primary/70" />
      {label ? (
        <p className="text-center text-[length:var(--text-xs)]">{label}</p>
      ) : (
        <span className="sr-only">Loading</span>
      )}
    </div>
  );
}
