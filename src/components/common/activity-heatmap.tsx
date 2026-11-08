import { useState } from "react";

import type { ActivityDay } from "../../types/storyquest";

const STATUS_COLOR: Record<string, string> = {
  productive: "bg-amber-400/30 border-amber-400/50 text-amber-400",
  vacation: "bg-purple-400/20 border-purple-400/40 text-purple-400",
  rest: "bg-slate-500/20 border-slate-500/40 text-slate-500",
  missed: "bg-white/[0.03] border-white/[0.06] text-slate-600",
};

const STATUS_LABEL: Record<string, string> = {
  productive: "Productive",
  vacation: "Vacation",
  rest: "Rest day",
  missed: "Missed",
};

interface ActivityHeatmapProps {
  days: ActivityDay[];
  onDayClick?: (day: ActivityDay) => void;
}

export function ActivityHeatmap({ days, onDayClick }: ActivityHeatmapProps) {
  const sorted = [...days].sort((a, b) => a.date.localeCompare(b.date));
  const [hovered, setHovered] = useState<string | null>(null);

  const maxXp = Math.max(1, ...sorted.map((d) => d.xp));
  const intensityFor = (xp: number) => Math.max(10, Math.min(100, (xp / maxXp) * 100 + 20));

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2 text-[10px] text-slate-500">
        {Object.entries(STATUS_COLOR).map(([status, color]) => (
          <div key={status} className="flex items-center gap-1.5">
            <span
              className={`h-2 w-2 rounded-full border ${color}`}
            />
            <span>{STATUS_LABEL[status]}</span>
          </div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1.5">
        {sorted.map((day) => {
          const isHovered = hovered === day.date;
          const baseColor = STATUS_COLOR[day.status] ?? STATUS_COLOR.missed;
          const intensity = intensityFor(day.xp);
          return (
            <div
              key={day.date}
              className={`aspect-square rounded-sm border cursor-pointer transition-all ${
                day.status === "productive"
                  ? `bg-amber-400/${Math.max(10, intensity / 100 * 80)} border-amber-400/${Math.max(20, intensity / 100 * 100)}`
                  : baseColor
              } ${isHovered ? "ring-2 ring-amber-400" : ""}`}
              onMouseEnter={() => setHovered(day.date)}
              onMouseLeave={() => setHovered(null)}
              onClick={() => onDayClick?.(day)}
              title={`${day.date}: ${STATUS_LABEL[day.status]} · ${day.xp} XP · ${day.tasksCompleted} tasks`}
            >
              <span className="absolute top-1 left-1 text-[9px] font-bold text-slate-600/60">
                {day.xp > 0 ? day.xp : ""}
              </span>
            </div>
          );
        })}
      </div>
      <div className="flex flex-wrap justify-end gap-2 text-[10px] text-slate-500">
        {sorted.map((day) => (
          <div key={day.date} className="flex items-center gap-1">
            <span
              className={`h-1.5 w-1.5 rounded-full border ${STATUS_COLOR[day.status] ?? STATUS_COLOR.missed}`}
            />
            <span>{day.date}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
