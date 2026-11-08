import type { LucideIcon } from "lucide-react";

import { Card, CardContent } from "../ui/card";
import { cn } from "../../lib/utils";

export function StatCard({
  label,
  value,
  hint,
  icon: Icon,
  accent = "gold",
}: {
  label: string;
  value: string;
  hint?: string;
  icon: LucideIcon;
  accent?: "gold" | "emerald" | "sky" | "rose";
}) {
  const accentClasses = {
    gold: "text-amber-300 bg-amber-400/10 border-amber-400/20",
    emerald: "text-emerald-300 bg-emerald-400/10 border-emerald-400/20",
    sky: "text-sky-300 bg-sky-400/10 border-sky-400/20",
    rose: "text-rose-300 bg-rose-400/10 border-rose-400/20",
  }[accent];

  return (
    <Card className="overflow-hidden">
      <CardContent className="flex items-start justify-between p-5">
        <div className="space-y-1.5">
          <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-500">
            {label}
          </p>
          <p className="text-2xl font-semibold tracking-tight text-slate-100">{value}</p>
          {hint ? <p className="text-xs leading-relaxed text-slate-500">{hint}</p> : null}
        </div>
        <div className={cn("rounded-xl border p-2.5", accentClasses)}>
          <Icon className="h-5 w-5" />
        </div>
      </CardContent>
    </Card>
  );
}
