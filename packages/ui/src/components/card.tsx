import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "../lib/cn";

const cardVariants = cva("border text-card-foreground transition-[background,box-shadow,border-color]", {
  variants: {
    variant: {
      default: "bg-card border-border shadow-sm rounded-[var(--radius-lg)]",
      glass: [
        "rounded-[var(--radius-xl)]",
        "border-[color:var(--glass-border)]",
        "bg-[color:var(--glass-bg)]",
        "shadow-[var(--glass-shadow)]",
        "backdrop-blur-[var(--blur-glass)]",
      ].join(" "),
      solid: "bg-card border-border rounded-[var(--radius-md)]",
      dialog: "bg-dialog text-dialog-foreground border-border shadow-lg rounded-[var(--radius-xl)]",
    },
    padding: {
      none: "p-0",
      sm: "p-3",
      md: "p-4",
      lg: "p-6",
    },
  },
  defaultVariants: {
    variant: "default",
    padding: "md",
  },
});

export interface CardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof cardVariants> {
  asChild?: boolean;
}

export function Card({ className, variant, padding, asChild = false, ...props }: CardProps) {
  const Comp = asChild ? Slot : "div";
  return <Comp className={cn(cardVariants({ variant, padding }), className)} {...props} />;
}

export function CardHeader({
  className,
  compact = false,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { compact?: boolean }) {
  return (
    <div
      className={cn(
        "flex flex-col gap-1.5",
        compact ? "px-4 pt-3 pb-0" : undefined,
        className,
      )}
      {...props}
    />
  );
}

export function CardTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h3
      className={cn("font-display text-(length:--text-lg) font-semibold leading-none", className)}
      {...props}
    />
  );
}

export function CardDescription({ className, ...props }: React.HTMLAttributes<HTMLParagraphElement>) {
  return (
    <p className={cn("text-(length:--text-sm) text-muted-foreground", className)} {...props} />
  );
}

export function CardContent({
  className,
  padding = "default",
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { padding?: "default" | "none" }) {
  return (
    <div
      className={cn(padding === "none" ? "p-0" : "pt-0", className)}
      {...props}
    />
  );
}

export function CardFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("flex items-center pt-4", className)} {...props} />;
}

export { cardVariants };
