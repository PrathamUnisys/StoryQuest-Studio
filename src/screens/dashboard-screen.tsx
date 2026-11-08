import {
  ArrowRight,
  Award,
  CalendarClock,
  CheckCircle2,
  Flame,
  ListChecks,
  PlayCircle,
  Sparkles,
  Youtube,
} from "lucide-react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { ActivityHeatmap } from "../components/common/activity-heatmap";
import { StatCard } from "../components/common/stat-card";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Progress } from "../components/ui/progress";
import { formatPercent } from "../lib/format";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

export function DashboardScreen() {
  const { bootstrap, setScreen, completeOnboarding, saving } = useStoryQuestStore();
  if (!bootstrap) return null;

  const {
    profile,
    creatorProgress,
    creatorRank,
    activeCampaign,
    activeEpisode,
    taskCounts,
    episodeProgressPercent,
    streak,
    achievements,
    weeklyActivity,
    youtube,
    onboardingCompleted,
  } = bootstrap;

  const nextRank = bootstrap.creatorRanks.find((rank) => rank.level > creatorRank.level);
  const closestAchievements = [...achievements]
    .filter((achievement) => !achievement.unlocked)
    .sort((a, b) => {
      const ratioA = a.progressTarget ? a.progressCurrent / a.progressTarget : 0;
      const ratioB = b.progressTarget ? b.progressCurrent / b.progressTarget : 0;
      return ratioB - ratioA;
    })
    .slice(0, 3);

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Studio overview"
        title={`Welcome back, ${profile.displayName.split(" ")[0] || "Creator"}`}
        description="Your production state, creator progression, and today's next step — all read from the local StoryQuest database."
        actions={
          <Button variant="secondary" onClick={() => setScreen("settings")}>
            Studio settings
          </Button>
        }
      />

      {!onboardingCompleted ? (
        <Card className="border-amber-400/20 bg-gradient-to-r from-amber-400/[0.075] via-transparent to-emerald-400/[0.035]">
          <CardContent className="flex flex-col gap-4 p-5 md:flex-row md:items-center md:justify-between">
            <div className="flex items-start gap-3">
              <div className="rounded-xl border border-amber-400/25 bg-amber-400/10 p-2 text-amber-300">
                <Sparkles className="h-5 w-5" />
              </div>
              <div>
                <p className="font-semibold text-slate-100">Finish setting up your studio</p>
                <p className="mt-1 text-sm text-slate-400">
                  Set your creator name and studio name, then confirm your starting rank. The
                  original demo progression has been cleared.
                </p>
              </div>
            </div>
            <div className="flex gap-2">
              <Button onClick={() => setScreen("settings")}>Open profile</Button>
              <Button
                variant="ghost"
                disabled={saving}
                onClick={() => void completeOnboarding()}
              >
                Mark complete
              </Button>
            </div>
          </CardContent>
        </Card>
      ) : null}

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <StatCard
          label="Creator rank"
          value={`${creatorRank.rankName}`}
          hint={`Rank ${creatorRank.level} of 13 · ${nextRank ? `next at ${nextRank.minPublishedVideos} published` : "highest rank reached"}`}
          icon={Award}
        />
        <StatCard
          label="Published videos"
          value={`${creatorProgress.publishedVideosTotal}`}
          hint={
            nextRank
              ? `${Math.max(0, nextRank.minPublishedVideos - creatorProgress.publishedVideosTotal)} to ${nextRank.rankName}`
              : "Legendary Creator"
          }
          icon={PlayCircle}
          accent="emerald"
        />
        <StatCard
          label="XP progression"
          value={`${profile.totalXp} XP`}
          hint={`Level ${profile.currentLevel} · ${profile.activeTitle}`}
          icon={Sparkles}
          accent="sky"
        />
        <StatCard
          label="Productive streak"
          value={`${streak.currentStreak} ${streak.currentStreak === 1 ? "day" : "days"}`}
          hint={`Best ${streak.bestStreak} · ${streak.streakFreezesAvailable} freeze${streak.streakFreezesAvailable === 1 ? "" : "s"} available`}
          icon={Flame}
          accent="rose"
        />
      </section>

      <div className="grid gap-6 xl:grid-cols-[1.55fr_1fr]">
        <div className="space-y-6">
          <Card>
            <CardHeader className="flex-row items-start justify-between space-y-0">
              <div>
                <CardTitle className="flex items-center gap-2">
                  Active production
                  {activeCampaign?.isSample ? <Badge variant="sample">Sample content</Badge> : null}
                </CardTitle>
                <CardDescription>
                  {activeCampaign
                    ? `${activeCampaign.genre} · ${activeCampaign.visualStyle}`
                    : "No production campaign yet."}
                </CardDescription>
              </div>
              <Button variant="ghost" size="sm" onClick={() => setScreen("campaigns")}>
                View
                <ArrowRight className="h-3.5 w-3.5" />
              </Button>
            </CardHeader>
            <CardContent className="space-y-5">
              {activeCampaign && activeEpisode ? (
                <>
                  <div>
                    <p className="font-display text-xl font-semibold text-slate-50">
                      {activeCampaign.title}
                    </p>
                    <p className="mt-1 text-sm text-slate-400">{activeEpisode.title}</p>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-4">
                    {[
                      { label: "Available", value: taskCounts.available, tone: "text-emerald-300" },
                      { label: "In progress", value: taskCounts.inProgress, tone: "text-amber-300" },
                      { label: "In review", value: taskCounts.review, tone: "text-sky-300" },
                      { label: "Completed", value: taskCounts.completed + taskCounts.approved, tone: "text-slate-200" },
                    ].map((item) => (
                      <div
                        key={item.label}
                        className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 px-3 py-2.5"
                      >
                        <p className="text-[10px] uppercase tracking-wider text-slate-600">
                          {item.label}
                        </p>
                        <p className={`mt-0.5 text-lg font-semibold ${item.tone}`}>{item.value}</p>
                      </div>
                    ))}
                  </div>

                  <div>
                    <div className="mb-2 flex items-center justify-between text-xs">
                      <span className="text-slate-500">Episode completion</span>
                      <span className="font-semibold text-slate-300">
                        {formatPercent(episodeProgressPercent)}
                      </span>
                    </div>
                    <Progress value={episodeProgressPercent} />
                    <p className="mt-2 text-xs text-slate-600">
                      Stage {activeEpisode.currentStage} of 15 · {taskCounts.total} production
                      tasks · {taskCounts.locked} locked
                    </p>
                  </div>

                  <div className="flex flex-wrap gap-2">
                    <Button size="sm" onClick={() => setScreen("quest-map")}>
                      Open Quest Map
                    </Button>
                    <Button variant="secondary" size="sm" onClick={() => setScreen("daily-quests")}>
                      <ListChecks className="h-3.5 w-3.5" />
                      Today&apos;s quests
                    </Button>
                  </div>
                </>
              ) : (
                <EmptyState
                  icon={Sparkles}
                  title="No active campaign"
                  description="Create a production campaign in Milestone 2. Until then, the studio shows an honest empty state instead of sample statistics."
                  badge="Milestone 2"
                />
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex-row items-start justify-between space-y-0">
              <div>
                <CardTitle>Today&apos;s quests</CardTitle>
                <CardDescription>
                  Daily quests are generated from your production tasks in Milestone 2.
                </CardDescription>
              </div>
              <Badge variant="muted">Not generated</Badge>
            </CardHeader>
            <CardContent>
              <EmptyState
                icon={ListChecks}
                title="No daily quests yet"
                description="The quest generator, completion flow, XP ledger, and streak updates arrive with the Daily Quest engine. No placeholder tasks are shown."
                badge="Milestone 2"
              />
            </CardContent>
          </Card>
        </div>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Youtube className="h-4 w-4 text-red-400" />
                YouTube channel
              </CardTitle>
              <CardDescription>{youtube.note}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                <p className="text-[10px] uppercase tracking-wider text-slate-600">Channel</p>
                <p className="mt-0.5 text-sm font-medium text-slate-300">
                  {youtube.channelTitle ?? "Not connected"}
                </p>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                  <p className="text-[10px] uppercase tracking-wider text-slate-600">Subscribers</p>
                  <p className="mt-0.5 text-sm font-medium text-slate-500">Unavailable</p>
                </div>
                <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                  <p className="text-[10px] uppercase tracking-wider text-slate-600">Watch time</p>
                  <p className="mt-0.5 text-sm font-medium text-slate-500">Unavailable</p>
                </div>
              </div>
              <Button variant="secondary" className="w-full" onClick={() => setScreen("youtube")}>
                Open YouTube analytics
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CalendarClock className="h-4 w-4 text-sky-300" />
                Weekly activity
              </CardTitle>
              <CardDescription>XP and completed tasks from the local ledger.</CardDescription>
            </CardHeader>
            <CardContent>
              <ActivityHeatmap days={weeklyActivity} />
              <p className="mt-3 text-xs text-slate-600">
                {weeklyActivity.some((day) => day.active)
                  ? "Activity is recorded from real XP events only."
                  : "No activity recorded in the last seven days."}
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex-row items-start justify-between space-y-0">
              <div>
                <CardTitle>Achievements</CardTitle>
                <CardDescription>Nearest locked achievements.</CardDescription>
              </div>
              <Button variant="ghost" size="sm" onClick={() => setScreen("achievements")}>
                All
              </Button>
            </CardHeader>
            <CardContent className="space-y-3">
              {closestAchievements.length ? (
                closestAchievements.map((achievement) => {
                  const progress = achievement.progressTarget
                    ? (achievement.progressCurrent / achievement.progressTarget) * 100
                    : 0;
                  return (
                    <div key={achievement.id} className="space-y-1.5">
                      <div className="flex items-center justify-between gap-2">
                        <p className="truncate text-sm font-medium text-slate-300">
                          {achievement.title}
                        </p>
                        <span className="shrink-0 text-xs text-slate-500">
                          {achievement.progressAvailable
                            ? `${achievement.progressCurrent}/${achievement.progressTarget}`
                            : "Manual"}
                        </span>
                      </div>
                      <Progress value={progress} className="h-1.5" />
                    </div>
                  );
                })
              ) : (
                <p className="text-sm text-slate-500">All achievements unlocked.</p>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CalendarClock className="h-4 w-4 text-sky-300" />
                Upcoming deadlines
              </CardTitle>
            </CardHeader>
            <CardContent>
              {bootstrap.deadlines.length ? (
                <div className="space-y-2">
                  {bootstrap.deadlines.map((task) => (
                    <div
                      key={task.id}
                      className="flex items-center justify-between rounded-lg border border-white/[0.06] bg-obsidian-900/60 px-3 py-2"
                    >
                      <span className="truncate text-sm text-slate-300">{task.title}</span>
                      <span className="ml-2 shrink-0 text-xs text-slate-500">{task.dueDate}</span>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="flex items-center gap-2 text-sm text-slate-500">
                  <CheckCircle2 className="h-4 w-4 text-emerald-400/70" />
                  No dated tasks.
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
