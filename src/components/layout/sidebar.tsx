import { useStoryQuestStore } from "../../store/useStoryQuestStore";
import { NAV_ITEMS } from "../../lib/navigation";
import { cn } from "../../lib/utils";
import { Badge } from "../ui/badge";
import { Progress } from "../ui/progress";

export function Sidebar() {
  const { activeScreen, setScreen, bootstrap } = useStoryQuestStore();

  return (
    <aside className="flex w-[272px] shrink-0 flex-col border-r border-white/[0.06] bg-obsidian-950/80 px-4 py-5 backdrop-blur">
      <div className="px-2">
        <div className="flex items-center gap-3">
          <div className="relative flex h-11 w-11 items-center justify-center rounded-2xl border border-amber-400/30 bg-gradient-to-br from-amber-300/20 to-emerald-400/10">
            <span className="font-display text-lg font-bold text-amber-300">SQ</span>
          </div>
          <div>
            <h1 className="font-display text-lg font-semibold leading-tight tracking-tight text-slate-50">
              StoryQuest
              <span className="ml-1 text-amber-400">Studio</span>
            </h1>
            <p className="text-[10px] leading-relaxed text-slate-500">
              Every episode is a quest.
            </p>
          </div>
        </div>
      </div>

      <div className="mt-6 flex-1 overflow-y-auto">
        <p className="mb-2 px-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-slate-600">
          Studio
        </p>
        <nav className="space-y-1">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const active = activeScreen === item.id;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => setScreen(item.id)}
                className={cn(
                  "group flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-medium transition",
                  active
                    ? "border border-amber-400/25 bg-amber-400/[0.09] text-amber-100 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]"
                    : "border border-transparent text-slate-400 hover:bg-white/[0.045] hover:text-slate-100"
                )}
              >
                <Icon
                  className={cn(
                    "h-[18px] w-[18px] shrink-0",
                    active ? "text-amber-300" : "text-slate-500 group-hover:text-slate-300"
                  )}
                />
                <span className="truncate">{item.label}</span>
                {active ? (
                  <span className="ml-auto h-1.5 w-1.5 rounded-full bg-amber-300 shadow-[0_0_10px_rgba(252,211,77,0.9)]" />
                ) : null}
              </button>
            );
          })}
        </nav>
      </div>

      <div className="mt-4 rounded-2xl border border-white/[0.07] bg-white/[0.025] p-3.5">
        {bootstrap ? (
          <>
            <div className="flex items-center justify-between gap-2">
              <Badge variant="default">Rank {bootstrap.creatorRank.level}</Badge>
              <span className="text-[10px] uppercase tracking-wider text-slate-600">
                Creator
              </span>
            </div>
            <p className="mt-2 truncate text-sm font-semibold text-slate-100">
              {bootstrap.creatorRank.rankName}
            </p>
            <p className="mt-0.5 truncate text-xs text-slate-500">
              {bootstrap.profile.displayName}
            </p>

            <div className="mt-3 space-y-1.5">
              <div className="flex items-center justify-between text-[10px] uppercase tracking-wider text-slate-500">
                <span>XP level {bootstrap.profile.currentLevel}</span>
                <span>{bootstrap.profile.totalXp} XP</span>
              </div>
              <Progress value={(bootstrap.profile.totalXp % 250) / 2.5} className="h-1.5" />
            </div>

            <div className="mt-3 grid grid-cols-2 gap-2">
              <div className="rounded-lg bg-obsidian-900/70 px-2 py-1.5">
                <p className="text-[10px] uppercase tracking-wider text-slate-600">Published</p>
                <p className="text-sm font-semibold text-slate-200">
                  {bootstrap.creatorProgress.publishedVideosTotal}
                </p>
              </div>
              <div className="rounded-lg bg-obsidian-900/70 px-2 py-1.5">
                <p className="text-[10px] uppercase tracking-wider text-slate-600">Streak</p>
                <p className="text-sm font-semibold text-slate-200">
                  {bootstrap.streak.currentStreak}d
                </p>
              </div>
            </div>
          </>
        ) : (
          <p className="text-xs text-slate-500">Loading studio state…</p>
        )}
      </div>
    </aside>
  );
}
