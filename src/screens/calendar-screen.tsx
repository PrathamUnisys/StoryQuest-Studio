import { Bell, CalendarClock, Clock, Loader2, Send } from "lucide-react";
import { useEffect, useState } from "react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { api } from "../lib/commands";
import { useStoryQuestStore } from "../store/useStoryQuestStore";
import type { ReminderConfig } from "../types/storyquest";

const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

function parseDays(json: string): string {
  try {
    const days = JSON.parse(json) as number[];
    if (!Array.isArray(days) || days.length === 0) return "Every day";
    return days.map((day) => DAY_LABELS[day] ?? day).join(", ");
  } catch {
    return "Every day";
  }
}

export function CalendarScreen() {
  const { bootstrap, notice, dismissNotice } = useStoryQuestStore();
  const [reminders, setReminders] = useState<ReminderConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      setReminders(await api.listReminders());
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const testToast = async () => {
    setSending(true);
    setToastMessage(null);
    try {
      await api.triggerTestToast(
        "StoryQuest Studio test",
        "Your desktop reminder channel is working.",
        25,
        25
      );
      setToastMessage("Test notification requested. Check the Windows notification center.");
    } catch (caught) {
      setToastMessage(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setSending(false);
    }
  };

  const today = bootstrap?.today ?? "";

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Schedule"
        title="Calendar"
        description="Daily and weekly production reminders. The reminder engine, Windows scheduled tasks, and quiet hours are completed in Milestone 5."
        actions={<Badge variant="muted">Milestone 5</Badge>}
      />

      {notice ? (
        <button
          type="button"
          onClick={dismissNotice}
          className="w-full rounded-xl border border-amber-400/20 bg-amber-400/[0.06] px-4 py-3 text-left text-sm text-amber-100"
        >
          {notice.message}
        </button>
      ) : null}

      {toastMessage ? (
        <div className="rounded-xl border border-sky-400/20 bg-sky-400/[0.05] px-4 py-3 text-sm text-sky-200">
          {toastMessage}
        </div>
      ) : null}

      <div className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
        <Card>
          <CardHeader className="flex-row items-start justify-between space-y-0">
            <div>
              <CardTitle>Reminder schedule</CardTitle>
              <CardDescription>Stored in the local database.</CardDescription>
            </div>
            <Button variant="secondary" size="sm" onClick={() => void load()} disabled={loading}>
              {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Clock className="h-3.5 w-3.5" />}
              Refresh
            </Button>
          </CardHeader>
          <CardContent className="space-y-3">
            {error ? <p className="text-sm text-red-300">{error}</p> : null}
            {!loading && reminders.length === 0 && !error ? (
              <EmptyState
                icon={Bell}
                title="No reminders configured"
                description="Reminder creation and Windows Task Scheduler sync are Milestone 5."
                badge="Milestone 5"
              />
            ) : (
              reminders.map((reminder) => (
                <div
                  key={reminder.id}
                  className="flex flex-col gap-3 rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3 sm:flex-row sm:items-center sm:justify-between"
                >
                  <div className="flex items-start gap-3">
                    <div className="rounded-lg border border-amber-400/20 bg-amber-400/[0.06] p-2 text-amber-300">
                      <Bell className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="text-sm font-medium text-slate-200">{reminder.title}</p>
                      <p className="mt-0.5 text-xs text-slate-500">
                        {reminder.scheduleTime} · {parseDays(reminder.daysOfWeekJson)}
                      </p>
                      <p className="mt-0.5 text-[11px] text-slate-600">
                        {reminder.reminderType} · {reminder.taskId ?? "No linked task"}
                      </p>
                    </div>
                  </div>
                  <Badge variant={reminder.enabled ? "success" : "muted"}>
                    {reminder.enabled ? "Enabled" : "Disabled"}
                  </Badge>
                </div>
              ))
            )}
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Test notification</CardTitle>
              <CardDescription>
                Verify Windows notifications before the reminder engine is built.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button className="w-full" onClick={() => void testToast()} disabled={sending}>
                {sending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Send className="h-4 w-4" />}
                Send test notification
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CalendarClock className="h-4 w-4 text-sky-300" />
                Today
              </CardTitle>
              <CardDescription>{today || "No date available"}</CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm leading-relaxed text-slate-500">
                No calendar events are scheduled. Reminder runs, quiet hours, snooze actions, and
                background execution are verified in Milestone 5.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
