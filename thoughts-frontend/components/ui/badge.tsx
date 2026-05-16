import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-primary text-primary-foreground hover:bg-primary/80 glossy-effect bottom text-shadow-sm",
        secondary:
          "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80 glossy-effect bottom text-shadow-sm",
        destructive:
          "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80 glossy-effect bottom text-shadow-sm",
        outline: "text-foreground glossy-effect bottom text-shadow-sm",
        branded:
          "border border-primary/20 bg-primary/8 text-primary font-semibold hover:bg-primary/15 hover:scale-105 transition-transform cursor-pointer",
        trending:
          "border border-red-300/30 bg-gradient-to-r from-orange-500/10 to-red-500/8 text-red-600 font-semibold hover:from-orange-500/18 hover:to-red-500/14 hover:scale-105 transition-transform cursor-pointer",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

function Badge({
  className,
  variant,
  asChild = false,
  ...props
}: React.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : "span";

  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
