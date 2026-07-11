import { motion, type HTMLMotionProps, type Transition } from "motion/react";

import { spring } from "../tokens/motion";

const diagonalReveal = {
  initial: { clipPath: "inset(0 100% 100% 0)" },
  animate: { clipPath: "inset(0 0 0 0)" },
  exit: { clipPath: "inset(0 100% 100% 0)" },
} as const;

export type DiagonalRevealProps = HTMLMotionProps<"div"> & {
  transition?: Transition;
};

export function DiagonalReveal({ className, transition, ...props }: DiagonalRevealProps) {
  return (
    <motion.div
      initial={diagonalReveal.initial}
      animate={diagonalReveal.animate}
      exit={diagonalReveal.exit}
      transition={transition ?? spring.snappy}
      className={className}
      {...props}
    />
  );
}
