import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "../../lib/cn";

const pageContainerVariants = cva("mx-auto w-full", {
  variants: {
    width: {
      full: "max-w-none",
      lg: "max-w-4xl",
      xl: "max-w-6xl",
    },
    padding: {
      none: "p-0",
      sm: "p-4",
      md: "p-6",
      lg: "p-8",
    },
  },
  defaultVariants: {
    width: "lg",
    padding: "md",
  },
});

export interface PageContainerProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof pageContainerVariants> {}

export function PageContainer({ className, width, padding, ...props }: PageContainerProps) {
  return <div className={cn(pageContainerVariants({ width, padding }), className)} {...props} />;
}

export { pageContainerVariants };
