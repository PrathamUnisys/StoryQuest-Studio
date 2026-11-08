import * as React from "react";

import { cn } from "../../lib/utils";

const Select = React.forwardRef<
  HTMLSelectElement,
  React.SelectHTMLAttributes<HTMLSelectElement>
>(({ className, children, ...props }, ref) => (
  <select
    ref={ref}
    className={cn(
      "focus-ring flex h-10 w-full appearance-none rounded-xl border border-white/10 bg-obsidian-900/80 px-3 py-2 text-sm text-slate-100",
      className
    )}
    {...props}
  >
    {children}
  </select>
));
Select.displayName = "Select";

const Switch = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement> & { checked: boolean }
>(({ className, checked, ...props }, ref) => (
  <button
    ref={ref}
    type="button"
    role="switch"
    aria-checked={checked}
    className={cn(
      "focus-ring relative h-6 w-11 shrink-0 rounded-full border transition-colors",
      checked
        ? "border-amber-400/40 bg-amber-400/30"
        : "border-white/10 bg-white/[0.05]",
      className
    )}
    {...props}
  >
    <span
      className={cn(
        "absolute top-[3px] h-[18px] w-[18px] rounded-full transition-all",
        checked ? "left-[22px] bg-amber-300" : "left-0.5 bg-slate-500"
      )}
    />
  </button>
));
Switch.displayName = "Switch";

export { Select, Switch };
