import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
  {
    variants: {
      variant: {
        // Default button gets blue gradient, gloss, and shadows
        default:
          "glass-effect fa-gradient-blue text-primary-foreground shadow-fa-md hover:bg-primary/90 active:shadow-fa-inner transition-transform active:scale-[0.98] glossy-effect",
        // Secondary gets green gradient
        secondary:
          "glass-effect fa-gradient-green text-secondary-foreground shadow-fa-md hover:bg-secondary/90 active:shadow-fa-inner transition-transform active:scale-[0.98] glossy-effect",
        // Ghost and Link should be more subtle
        ghost:
          "glass-effect hover:bg-accent hover:text-accent-foreground rounded-lg", // Keep them simple, maybe a slight blur/gloss on hover
        link: "text-primary underline-offset-4 hover:underline",
        // Outline button for a transparent-ish, glassy feel
        outline:
          "border border-input bg-background/80 hover:bg-accent/80 hover:text-accent-foreground backdrop-blur-sm shadow-fa-sm glossy-effect",
        destructive:
          "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean;
  }) {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
