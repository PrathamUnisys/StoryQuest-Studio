import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { Badge } from "../ui/badge";

export function EmptyState({
  icon: Icon,
  title,
  description,
  badge,
  action,
}: {
  icon: LucideIcon;
  title: string;
  description: string;
  badge?: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-white/10 bg-white/[0.015] px-6 py-14 text-center">
      <div className="mb-4 rounded-2xl border border-amber-400/15 bg-amber-400/[0.06] p-3.5 text-amber-300">
        <Icon className="h-6 w-6" />
      </div>
      {badge ? (
        <Badge variant="muted" className="mb-3">
          {badge}
        </Badge>
      ) : null}
      <h3 className="font-display text-lg font-semibold text-slate-200">{title}</h3>
      <p className="mt-2 max-w-md text-sm leading-relaxed text-slate-500">{description}</p>
      {action ? <div className="mt-5">{action}</div> : null}
    </div>
  );
}
