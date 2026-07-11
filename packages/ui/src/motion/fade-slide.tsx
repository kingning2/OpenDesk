import { motion, type HTMLMotionProps } from "motion/react";

import { spring } from "../tokens/motion";

export function FadeSlide({ children, className, ...props }: HTMLMotionProps<"div">) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={spring.smooth}
      className={className}
      {...props}
    >
      {children}
    </motion.div>
  );
}
