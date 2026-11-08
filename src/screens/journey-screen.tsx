import { Award, CalendarClock, Eye, Film, Flag, Flame, PlayCircle, Sparkles, Users } from "lucide-react";

import { PageHeader } from "../components/common/page-header";
import { ActivityHeatmap } from "../components/common/activity-heatmap";
import { StatCard } from "../components/common/stat-card";
import { Badge } from "../components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Progress } from "../components/ui/progress";
import { formatNumber } from "../lib/format";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

export function JourneyScreen() {
  const { bootstrap } = useStoryQuestStore();
  if (!bootstrap) return null;

  const {
    creatorRank,
    creatorRanks,
    creatorProgress,
    profile,
    achievements,
    streak,
    weeklyActivity,
  } = bootstrap;
  const nextRank = creatorRanks.find((rank) => rank.level > creatorRank.level);
  const nextTarget = nextRank?.minPublishedVideos ?? creatorRank.minPublishedVideos;
  const range = Math.max(1, nextTarget - creatorRank.minPublishedVideos);
  const progressInRank = nextRank
    ? ((creatorProgress.publishedVideosTotal - creatorRank.minPublishedVideos) / range) * 100
    : 100;
  const unlockedCount = achievements.filter((achievement) => achievement.unlocked).length;

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Creator progression"
        title="My Journey"
        description="Creator ranks are based on published videos, not XP. YouTube metrics stay unavailable until the channel is connected with the official APIs."
      />

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard
          label="Current rank"
          value={creatorRank.rankName}
          hint={`${creatorRank.minPublishedVideos}+ published videos`}
          icon={Award}
        />
        <StatCard
          label="Published videos"
          value={`${creatorProgress.publishedVideosTotal}`}
          hint={
            nextRank
              ? `${nextTarget - creatorProgress.publishedVideosTotal} more to ${nextRank.rankName}`
              : "Highest creator rank"
          }
          icon={PlayCircle}
          accent="emerald"
        />
        <StatCard
          label="XP level"
          value={`Level ${profile.currentLevel}`}
          hint={`${profile.totalXp} XP · ${profile.activeTitle}`}
          icon={Sparkles}
          accent="sky"
        />
        <StatCard
          label="Achievements"
          value={`${unlockedCount}/${achievements.length}`}
          hint="Locked achievements use real progress where available"
          icon={Film}
          accent="rose"
        />
        <StatCard
          label="Current streak"
          value={`${streak.currentStreak} day${streak.currentStreak === 1 ? "" : "s"}`}
          hint={`Best ${streak.bestStreak} · ${streak.streakFreezesAvailable} freeze${streak.streakFreezesAvailable === 1 ? "" : "s"}`}
          icon={Flame}
          accent="rose"
        />
      </section>

      <div className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Creator rank path</CardTitle>
            <CardDescription>
              Each rank unlocks at a published-video milestone. Thresholds are editable in
              Settings.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="mb-5 rounded-xl border border-amber-400/15 bg-amber-400/[0.04] p-4">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div>
                  <p className="text-[10px] uppercase tracking-wider text-amber-400/70">
                    Rank {creatorRank.level}
                  </p>
                  <p className="font-display text-xl font-semibold text-slate-50">
                    {creatorRank.rankName}
                  </p>
                </div>
                {nextRank ? (
                  <Badge variant="secondary">
                    Next: {nextRank.rankName} at {nextRank.minPublishedVideos}
                  </Badge>
                ) : (
                  <Badge variant="default">Maximum rank</Badge>
                )}
              </div>
              <p className="mt-2 text-sm leading-relaxed text-slate-400">
                {creatorRank.description}
              </p>
              <div className="mt-3">
                <Progress value={progressInRank} />
              </div>
            </div>

            <div className="space-y-2">
              {creatorRanks.map((rank) => {
                const reached = creatorProgress.publishedVideosTotal >= rank.minPublishedVideos;
                const current = rank.level === creatorRank.level;
                return (
                  <div
                    key={rank.level}
                    className={`flex items-center gap-3 rounded-xl border px-3 py-2.5 ${
                      current
                        ? "border-amber-400/25 bg-amber-400/[0.06]"
                        : reached
                          ? "border-emerald-400/15 bg-emerald-400/[0.025]"
                          : "border-white/[0.05] bg-white/[0.015]"
                    }`}
                  >
                    <div
                      className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full border text-xs font-bold ${
                        reached
                          ? "border-emerald-400/30 bg-emerald-400/10 text-emerald-300"
                          : "border-white/10 bg-white/[0.03] text-slate-600"
                      }`}
                    >
                      {rank.level}
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <p
                          className={`truncate text-sm font-medium ${
                            reached ? "text-slate-200" : "text-slate-500"
                          }`}
                        >
                          {rank.rankName}
                        </p>
                        {current ? <Badge variant="default">Current</Badge> : null}
                      </div>
                      <p className="text-xs text-slate-600">{rank.description}</p>
                    </div>
                    <div className="shrink-0 text-right">
                      <p className="text-sm font-semibold text-slate-300">
                        {rank.minPublishedVideos}
                      </p>
                      <p className="text-[10px] uppercase tracking-wider text-slate-600">
                        videos
                      </p>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CalendarClock className="h-4 w-4 text-sky-300" />
                Weekly activity heatmap
              </CardTitle>
              <CardDescription>XP and task activity from the local ledger.</CardDescription>
            </CardHeader>
            <CardContent>
              <ActivityHeatmap days={weeklyActivity} />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Eye className="h-4 w-4 text-slate-400" />
                Streak detail
              </CardTitle>
              <CardDescription>Rest days and vacation mode never deduct XP.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex items-center justify-between text-sm">
                <span className="text-slate-500">Current streak</span>
                <span className="font-semibold text-slate-300">
                  {streak.currentStreak} day{streak.currentStreak === 1 ? "" : "s"}
                </span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-slate-500">Best streak</span>
                <span className="font-semibold text-slate-300">{streak.bestStreak}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-slate-500">Freezes available</span>
                <span className="font-semibold text-slate-300">
                  {streak.streakFreezesAvailable}
                </span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-slate-500">Rest day</span>
                <span className="font-semibold text-slate-300">
                  {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][streak.weeklyRestDay]}
                </span>
              </div>
              {streak.vacationMode && streak.vacationUntil ? (
                <div className="flex items-center justify-between text-sm text-purple-300">
                  <span className="text-slate-500">Vacation until</span>
                  <span className="font-semibold">{streak.vacationUntil}</span>
                </div>
              ) : null}
              <p className="pt-2 text-xs text-slate-600">
                Eligible missed days can be bridged with freezes (up to 2 consecutive days).
                Vacation days and rest days never break the streak.
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Channel statistics</CardTitle>
              <CardDescription>
                Values appear only after an official YouTube connection and sync.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {[
                {
                  icon: Users,
                  label: "Subscribers",
                  value: creatorProgress.subscriberCount,
                },
                {
                  icon: Eye,
                  label: "Total views",
                  value: creatorProgress.totalChannelViews,
                },
                {
                  icon: Film,
                  label: "Watch minutes",
                  value: creatorProgress.totalWatchMinutes,
                },
              ].map((item) => (
                <div
                  key={item.label}
                  className="flex items-center justify-between rounded-xl border border-white/[0.06] bg-obsidian-900/60 px-3 py-2.5"
                >
                  <div className="flex items-center gap-2 text-sm text-slate-400">
                    <item.icon className="h-4 w-4 text-slate-600" />
                    {item.label}
                  </div>
                  <span className="text-sm font-medium text-slate-300">
                    {item.value === null || item.value === undefined
                      ? "Unavailable"
                      : formatNumber(item.value)}
                  </span>
                </div>
              ))}
              <p className="text-xs leading-relaxed text-slate-600">
                No estimates are generated. YouTube Analytics API data is added in Milestone 4.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
