import { ListChecks, Timer, Play, Loader2, RefreshCw } from "lucide-react";
import { useState, useEffect } from "react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Button } from "../components/ui/button";
import { api } from "../lib/commands";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

export function DailyQuestsScreen() {
  const {
    bootstrap,
    setTaskStatus,
    reopenTask,
    loadBootstrap,
  } = useStoryQuestStore();
  const [quests, setQuests] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);

  const episodeId = bootstrap?.activeEpisode?.id;

  const load = async () => {
    if (!episodeId) {
      setQuests([]);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const tasks = await api.listEpisodeTasks(episodeId);
      // Today's quests: available + in_progress tasks from the active episode
      setQuests(tasks.filter((t) => t.status === "available" || t.status === "in_progress"));
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, [episodeId]);

  const setStatus = async (task: any, status: string) => {
    setSaving((prev) => ({ ...prev, [task.id]: true }));
    setError(null);
    try {
      await api.setTaskStatus({ taskId: task.id, status });
      await loadBootstrap();
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
      <div className="animate-fade-in space-y-6">
        <PageHeader
          eyebrow="Daily loop"
          title="Daily Quests"
          description="Short, specific production tasks generated from the active roadmap."
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
        eyebrow="Daily loop"
        title="Daily Quests"
        description={`Today's quests for ${bootstrap.activeEpisode.title} — pick an available task, start it, and complete it to earn XP.`}
        actions={
          <>
            <Button variant="secondary" onClick={() => void load()} disabled={loading}>
              {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
              Refresh
            </Button>
          </>
        }
      />

      <div className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardContent className="p-4">
            <p className="text-[10px] uppercase tracking-wider text-slate-600">Today's quests</p>
            <p className="mt-1 text-lg font-semibold text-slate-200">{quests.length}</p>
            <p className="mt-1 text-xs text-slate-600">Available or in progress</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-[10px] uppercase tracking-wider text-slate-600">XP potential</p>
            <p className="mt-1 text-lg font-semibold text-slate-200">
              {quests.reduce((sum, t) => sum + (t.xpReward || 0), 0)}
            </p>
            <p className="mt-1 text-xs text-slate-600">Sum of today's quest rewards</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-[10px] uppercase tracking-wider text-slate-600">Focus minimum</p>
            <p className="mt-1 text-lg font-semibold text-slate-200">15 min</p>
            <p className="mt-1 text-xs text-slate-600">Configurable in Settings</p>
          </CardContent>
        </Card>
      </div>

      {error ? (
        <Card>
          <CardContent className="p-5 text-sm text-red-300">{error}</CardContent>
        </Card>
      ) : null}

      {quests.length === 0 && !error && !loading ? (
        <Card>
          <CardContent>
            <EmptyState
              icon={ListChecks}
              title="No quests today"
              description="All tasks for this episode are locked or already completed. Advance stages in the Quest Map or create a new episode."
              badge="Milestone 2"
            />
          </CardContent>
        </Card>
      ) : null}

      {quests.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Today's quest log</CardTitle>
            <CardDescription>
              Start a quest to begin work. Complete it to earn XP and unlock dependent tasks.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {quests.map((task) => {
              const isSaving = saving[task.id];
              const isInProgress = task.status === "in_progress";
              return (
                <Card
                  key={task.id}
                  className={isInProgress ? "border-blue-400/25 bg-blue-400/[0.04]" : ""}
                >
                  <CardContent className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <p className="text-sm font-semibold text-slate-200">{task.title}</p>
                        <Badge variant="default">
                          Stage {task.stageNumber} · {task.difficulty}
                        </Badge>
                        {task.isDailyQuest ? <Badge variant="default" className="bg-amber-500/10 text-amber-400 border-amber-500/20">Daily</Badge> : null}
                      </div>
                      <p className="mt-1 text-xs leading-relaxed text-slate-500">
                        {task.description}
                      </p>
                      <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-[11px] text-slate-600">
                        <span>{task.skillCategory}</span>
                        <span>{task.xpReward} XP</span>
                        <span>{task.estimatedEffortMins} min est.</span>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      {isInProgress && !isSaving ? (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => void setStatus(task, "paused")}
                          disabled={loading}
                        >
                          <Play className="h-3 w-3" /> Pause
                        </Button>
                      ) : !isInProgress && !isSaving ? (
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => void setStatus(task, "in_progress")}
                          disabled={loading}
                        >
                          <Play className="h-3 w-3" /> Start
                        </Button>
                      ) : (
                        <Button variant="secondary" size="sm" disabled>
                          <Loader2 className="h-3 w-3 animate-spin" />
                        </Button>
                      )}
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
