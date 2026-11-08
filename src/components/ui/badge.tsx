import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

const badgeVariants = cva(
  "inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-[11px] font-semibold uppercase tracking-wider",
  {
    variants: {
      variant: {
        default: "border-amber-400/25 bg-amber-400/10 text-amber-200",
        secondary: "border-white/10 bg-white/[0.04] text-slate-300",
        success: "border-emerald-400/25 bg-emerald-400/10 text-emerald-300",
        muted: "border-white/[0.06] bg-white/[0.03] text-slate-500",
        destructive: "border-red-400/25 bg-red-400/10 text-red-300",
        sample: "border-sky-400/25 bg-sky-400/10 text-sky-300",
      },
    },
    defaultVariants: { variant: "default" },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { Badge, badgeVariants };
