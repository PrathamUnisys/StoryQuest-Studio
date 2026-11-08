import { Award, Lock, Trophy } from "lucide-react";
import { useState } from "react";

import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Progress } from "../components/ui/progress";
import { formatDate } from "../lib/format";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

type Filter = "all" | "unlocked" | "locked";

export function AchievementsScreen() {
  const { bootstrap } = useStoryQuestStore();
  const [filter, setFilter] = useState<Filter>("all");
  if (!bootstrap) return null;

  const unlocked = bootstrap.achievements.filter((achievement) => achievement.unlocked);
  const filtered = bootstrap.achievements.filter((achievement) => {
    // Hidden achievements stay invisible until unlocked.
    if (achievement.isHidden && !achievement.unlocked) return false;
    if (filter === "unlocked") return achievement.unlocked;
    if (filter === "locked") return !achievement.unlocked;
    return true;
  });

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Milestones"
        title="Creator Achievements"
        description="Production achievements use real task and episode data. YouTube achievements stay locked until a channel is connected and synced."
      />

      <div className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardContent className="p-5">
            <p className="text-[10px] uppercase tracking-wider text-slate-600">Unlocked</p>
            <p className="mt-1 text-2xl font-semibold text-emerald-300">{unlocked.length}</p>
            <p className="mt-1 text-xs text-slate-600">of {bootstrap.achievements.length}</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-5">
            <p className="text-[10px] uppercase tracking-wider text-slate-600">Locked</p>
            <p className="mt-1 text-2xl font-semibold text-slate-300">
              {bootstrap.achievements.length - unlocked.length}
            </p>
            <p className="mt-1 text-xs text-slate-600">Real progress shown where available</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-5">
            <p className="text-[10px] uppercase tracking-wider text-slate-600">Demo unlocks cleared</p>
            <p className="mt-1 text-2xl font-semibold text-slate-300">0</p>
            <p className="mt-1 text-xs text-slate-600">No fabricated achievements</p>
          </CardContent>
        </Card>
      </div>

      <div className="flex flex-wrap gap-2">
        {(["all", "unlocked", "locked"] as Filter[]).map((option) => (
          <Button
            key={option}
            size="sm"
            variant={filter === option ? "default" : "secondary"}
            onClick={() => setFilter(option)}
            className="capitalize"
          >
            {option}
          </Button>
        ))}
      </div>

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {filtered.map((achievement) => {
          const progress = achievement.progressTarget
            ? Math.min(100, (achievement.progressCurrent / achievement.progressTarget) * 100)
            : 0;
          return (
            <Card
              key={achievement.id}
              className={achievement.unlocked ? "border-emerald-400/20" : undefined}
            >
              <CardHeader className="space-y-3">
                <div className="flex items-start justify-between gap-2">
                  <div
                    className={`rounded-xl border p-2.5 ${
                      achievement.unlocked
                        ? "border-emerald-400/25 bg-emerald-400/10 text-emerald-300"
                        : "border-white/[0.07] bg-white/[0.03] text-slate-500"
                    }`}
                  >
                    {achievement.unlocked ? (
                      <Trophy className="h-5 w-5" />
                    ) : (
                      <Lock className="h-5 w-5" />
                    )}
                  </div>
                  <Badge variant={achievement.unlocked ? "success" : "muted"}>
                    {achievement.category}
                  </Badge>
                </div>
                <div>
                  <CardTitle className="text-base">{achievement.title}</CardTitle>
                  <CardDescription className="mt-1">{achievement.description}</CardDescription>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-600">
                    {achievement.progressAvailable
                      ? `${achievement.progressCurrent} / ${achievement.progressTarget}`
                      : "Manual milestone"}
                  </span>
                  <span className="text-amber-300">+{achievement.xpBonus} XP</span>
                </div>
                <Progress value={achievement.progressAvailable ? progress : 0} className="h-1.5" />
                <div className="flex items-center justify-between text-[11px] text-slate-600">
                  <span className="flex items-center gap-1">
                    <Award className="h-3 w-3" />
                    {achievement.cosmeticReward ?? "No cosmetic reward"}
                  </span>
                  <span>
                    {achievement.unlocked && achievement.unlockedAt
                      ? formatDate(achievement.unlockedAt)
                      : "Locked"}
                  </span>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
