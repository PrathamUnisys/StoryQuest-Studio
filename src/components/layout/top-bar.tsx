import { CalendarDays, Flame, PlayCircle, Youtube } from "lucide-react";

import { screenTitle } from "../../lib/navigation";
import { useStoryQuestStore } from "../../store/useStoryQuestStore";
import { Badge } from "../ui/badge";

export function TopBar() {
  const { activeScreen, bootstrap } = useStoryQuestStore();

  return (
    <header className="flex h-16 shrink-0 items-center justify-between gap-4 border-b border-white/[0.06] bg-obsidian-950/55 px-6 backdrop-blur lg:px-8">
      <div>
        <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-slate-600">
          StoryQuest Studio
        </p>
        <h2 className="text-sm font-semibold text-slate-200">{screenTitle(activeScreen)}</h2>
      </div>

      <div className="flex items-center gap-2">
        {bootstrap ? (
          <>
            <div className="hidden items-center gap-1.5 rounded-full border border-white/[0.07] bg-white/[0.03] px-3 py-1.5 text-xs text-slate-400 lg:flex">
              <CalendarDays className="h-3.5 w-3.5 text-slate-500" />
              {bootstrap.today}
            </div>
            <div className="flex items-center gap-1.5 rounded-full border border-emerald-400/15 bg-emerald-400/[0.06] px-3 py-1.5 text-xs text-emerald-300">
              <Flame className="h-3.5 w-3.5" />
              {bootstrap.streak.currentStreak} day streak
            </div>
            <div className="flex items-center gap-1.5 rounded-full border border-amber-400/15 bg-amber-400/[0.06] px-3 py-1.5 text-xs text-amber-200">
              <PlayCircle className="h-3.5 w-3.5" />
              {bootstrap.creatorProgress.publishedVideosTotal} published
            </div>
            <Badge variant={bootstrap.youtube.connected ? "success" : "muted"}>
              <Youtube className="h-3 w-3" />
              {bootstrap.youtube.connected ? "Connected" : "Not connected"}
            </Badge>
          </>
        ) : null}
      </div>
    </header>
  );
}
