import { Compass, Loader2, Lock, Play, Pause, RefreshCw, RotateCcw, Check, X } from "lucide-react";
import { useEffect, useState } from "react";

import { Button, type ButtonProps } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Progress } from "../components/ui/progress";
import { Separator } from "../components/ui/separator";
import { api } from "../lib/commands";
import { useStoryQuestStore } from "../store/useStoryQuestStore";
import type { QuestTask } from "../types/storyquest";

const STATUS_LABEL: Record<string, string> = {
  locked: "Locked",
  blocked: "Blocked",
  available: "Available",
  in_progress: "In progress",
  paused: "Paused",
  review: "In review",
  approved: "Approved",
  completed: "Completed",
};

const STATUS_COLOR: Record<string, string> = {
  locked: "border-white/[0.06] bg-white/[0.02] text-slate-600",
  blocked: "border-white/[0.06] bg-white/[0.02] text-slate-600",
  available: "border-amber-400/25 bg-amber-400/[0.08] text-amber-300",
  in_progress: "border-blue-400/25 bg-blue-400/[0.08] text-blue-300",
  paused: "border-slate-500/25 bg-slate-500/[0.08] text-slate-400",
  review: "border-purple-400/25 bg-purple-400/[0.08] text-purple-300",
  approved: "border-emerald-400/25 bg-emerald-400/[0.08] text-emerald-300",
  completed: "border-emerald-500/30 bg-emerald-500/[0.12] text-emerald-400",
};

const ACTION_BY_STATUS: Record<string, { label: string; icon: React.ElementType; variant: string; nextStatus: string }[]> = {
  available: [{ label: "Start", icon: Play, variant: "secondary", nextStatus: "in_progress" }],
  in_progress: [
    { label: "Pause", icon: Pause, variant: "outline", nextStatus: "paused" },
    { label: "Submit review", icon: Check, variant: "success", nextStatus: "review" },
  ],
  paused: [{ label: "Resume", icon: Play, variant: "secondary", nextStatus: "in_progress" }],
  review: [{ label: "Approve", icon: Check, variant: "success", nextStatus: "approved" }],
  approved: [{ label: "Complete", icon: Check, variant: "success", nextStatus: "completed" }],
};

export function QuestMapScreen() {
  const {
    bootstrap,
    setScreen,
    setTaskStatus,
    reopenTask,
  } = useStoryQuestStore();
  const [tasks, setTasks] = useState<QuestTask[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);

  const episodeId = bootstrap?.activeEpisode?.id;

  const load = async () => {
    if (!episodeId) return;
    setLoading(true);
    setError(null);
    try {
      setTasks(await api.listEpisodeTasks(episodeId));
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, [episodeId]);

  const setStatus = async (task: QuestTask, status: string) => {
    setSaving((prev) => ({ ...prev, [task.id]: true }));
    setError(null);
    try {
      await setTaskStatus({ taskId: task.id, status });
      void load();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setSaving((prev) => {
        const next = { ...prev };
        delete next[task.id];
        return next;
      });
    }
  };

  if (!bootstrap?.activeCampaign || !bootstrap.activeEpisode) {
    return (
      <div className="animate-fade-in">
        <PageHeader
          eyebrow="Production roadmap"
          title="Quest Map"
          description="The 15-stage animated production roadmap for the active episode."
        />
        <Card>
          <CardContent className="p-5 text-sm text-slate-500">
            No active campaign or episode. Create one in Production Campaigns.
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Production roadmap"
        title="Quest Map"
        description={`${bootstrap.activeCampaign.title} · ${bootstrap.activeEpisode.title}`}
        actions={
          <>
            <Button variant="secondary" onClick={() => void load()} disabled={loading}>
              {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
              Refresh
            </Button>
            <Button variant="secondary" onClick={() => setScreen("daily-quests")}>
              Daily quests
            </Button>
          </>
        }
      />

      <Card>
        <CardContent className="flex flex-col gap-4 p-5 md:flex-row md:items-center md:justify-between">
          <div>
  <p className="text-sm text-slate-400">
    Stage {bootstrap.activeEpisode.currentStage} of 15 ·{" "}
              {bootstrap.taskCounts.available} available · {bootstrap.taskCounts.locked} locked
            </p>
            <div className="mt-2 w-full max-w-md">
              <Progress value={bootstrap.episodeProgressPercent} />
            </div>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={() => setScreen("campaigns")}>
              Campaigns
            </Button>
          </div>
        </CardContent>
      </Card>

      {error ? (
        <Card>
          <CardContent className="p-5 text-sm text-red-300">{error}</CardContent>
        </Card>
      ) : null}

      <div className="space-y-3">
        {(tasks.length ? tasks : []).map((task) => {
          const isLocked = task.status === "locked" || task.status === "blocked";
          const action = ACTION_BY_STATUS[task.status];
          const isSaving = saving[task.id];
          return (
            <Card
              key={task.id}
              className={isLocked ? "opacity-75" : "border-amber-400/15"}
            >
              <CardContent className="flex flex-col gap-4 p-4 sm:flex-row sm:items-center">
                <div
                  className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border text-sm font-bold ${
                    isLocked
                      ? "border-white/[0.06] bg-white/[0.02] text-slate-600"
                      : STATUS_COLOR[task.status] ?? STATUS_COLOR.available
                  }`}
                >
                  {isLocked ? <Lock className="h-4 w-4" /> : task.stageNumber}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="text-sm font-semibold text-slate-200">{task.title}</p>
                    <Badge
                      variant={
                        task.status === "available"
                          ? "success"
                          : task.status === "locked"
                            ? "muted"
                            : "secondary"
                      }
                    >
                      {STATUS_LABEL[task.status] ?? task.status}
                    </Badge>
                    {task.isDailyQuest ? <Badge variant="default">Daily</Badge> : null}
                  </div>
                  <p className="mt-1 line-clamp-2 text-xs leading-relaxed text-slate-500">
                    {task.description}
                  </p>
                  <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-[11px] text-slate-600">
                    <span>{task.skillCategory}</span>
                    <span>{task.estimatedEffortMins} min est.</span>
                    <span>{task.difficulty}</span>
                    <span>{task.xpReward} XP</span>
                    <span>{task.dependencies.length} dependencies</span>
                  </div>
                </div>
                <div className="shrink-0 flex flex-col gap-2 sm:flex-row sm:items-end">
                  <div className="text-right">
                    <p className="text-[10px] uppercase tracking-wider text-slate-600">Stage</p>
                    <p className="text-sm font-semibold text-slate-300">{task.stageNumber}/15</p>
                  </div>
                  <div className="flex gap-2">
                    {Array.isArray(action) ? (
                      action.map((act, idx) => (
                        <Button
                          key={idx}
                          variant={act.variant as ButtonProps["variant"]}
                          size="sm"
                          onClick={() => void setStatus(task, act.nextStatus)}
                          disabled={loading || isSaving}
                        >
                          {isSaving ? <Loader2 className="h-3 w-3 animate-spin" /> : act.icon === Play ? <Play className="h-3 w-3" /> : act.icon === Pause ? <Pause className="h-3 w-3" /> : act.icon === Check ? <Check className="h-3 w-3" /> : <RotateCcw className="h-3 w-3" />}{" "}
                          {act.label}
                        </Button>
                      ))
                    ) : null}
                  </div>
                </div>
              </CardContent>
              {task.status === "completed" && (
                <CardFooter className="flex justify-end">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => void reopenTask({ taskId: task.id })}
                    disabled={loading || saving[task.id]}
                  >
                    <X className="h-3 w-3" /> Reopen task
                  </Button>
                </CardFooter>
              )}
            </Card>
          );
        })}
      </div>

      {!loading && tasks.length === 0 && !error ? (
        <Card>
          <CardContent className="flex items-center gap-2 p-5 text-sm text-slate-500">
            <Compass className="h-4 w-4" />
            No tasks found for this episode.
          </CardContent>
        </Card>
      ) : null}

      <Separator className="my-6" />

      <Card className="border-dashed">
        <CardHeader>
          <CardTitle className="text-slate-300 flex items-center gap-2">
            <Check className="h-4 w-4 text-slate-500" /> How task progression works
          </CardTitle>
          <CardDescription className="text-sm text-slate-400">
            Available → Start (in progress) → Submit for review → Approve → Complete. A task can only
            be completed once all its dependencies are satisfied. Completing a task awards its XP and
            unlocks dependent tasks. Completed tasks can be reopened to "available".
          </CardDescription>
        </CardHeader>
      </Card>
    </div>
  );
}
