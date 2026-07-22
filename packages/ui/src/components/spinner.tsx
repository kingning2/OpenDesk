/**
 * Animated loading spinner for buttons, lists, and panels.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { cva, type VariantProps } from "class-variance-authority";
import { Loader2 } from "lucide-react";
import type { SVGAttributes } from "react";

import { cn } from "../lib/cn";

const spinnerVariants = cva("animate-spin text-muted-foreground", {
  variants: {
    size: {
      sm: "size-3.5",
      md: "size-5",
      lg: "size-7",
    },
  },
  defaultVariants: {
    size: "md",
  },
});

export interface SpinnerProps extends SVGAttributes<SVGSVGElement>, VariantProps<typeof spinnerVariants> {}

/**
 * Rotating loader icon.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param props.size - Visual size (`sm` | `md` | `lg`)
 * @param props.className - Extra class names
 * @returns Spinner SVG node
 */
export function Spinner({ className, size, ...props }: SpinnerProps) {
  return (
    <Loader2 aria-hidden="true" className={cn(spinnerVariants({ size }), className)} {...props} />
  );
}

export { spinnerVariants };
