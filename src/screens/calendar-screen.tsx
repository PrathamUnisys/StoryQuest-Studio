import { Bell, CalendarClock, Clock, Loader2, Pencil, Plus, Send, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { api } from "../lib/commands";
import { useStoryQuestStore } from "../store/useStoryQuestStore";
import type { ReminderConfig } from "../types/storyquest";

const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const REMINDER_TYPES = ["daily", "weekly", "specific_date", "task_deadline", "weekly_review", "custom"];

function parseDays(json: string): string {
  try {
    const days = JSON.parse(json) as number[];
    if (!Array.isArray(days) || days.length === 0) return "Every day";
    return days.map((day) => DAY_LABELS[day] ?? day).join(", ");
  } catch {
    return "Every day";
  }
}

function parseDaysArray(json: string): number[] {
  try {
    const days = JSON.parse(json) as number[];
    return Array.isArray(days) ? days : [1, 2, 3, 4, 5];
  } catch {
    return [1, 2, 3, 4, 5];
  }
}

interface FormState {
  id: string | null;
  title: string;
  reminderType: string;
  scheduleTime: string;
  days: number[];
  specificDate: string;
  enabled: boolean;
}

const EMPTY_FORM: FormState = {
  id: null,
  title: "",
  reminderType: "daily",
  scheduleTime: "09:00",
  days: [1, 2, 3, 4, 5],
  specificDate: "",
  enabled: true,
};

export function CalendarScreen() {
  const { bootstrap, notice, dismissNotice } = useStoryQuestStore();
  const [reminders, setReminders] = useState<ReminderConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [saving, setSaving] = useState(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [editing, setEditing] = useState(false);

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
      setToastMessage("Test notification sent (or suppressed by quiet hours — see message).");
    } catch (caught) {
      setToastMessage(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setSending(false);
    }
  };

  const toggleDay = (day: number) =>
    setForm((f) => ({ ...f, days: f.days.includes(day) ? f.days.filter((d) => d !== day) : [...f.days, day] }));

  const save = async () => {
    setSaving(true);
    setToastMessage(null);
    try {
      const payload = {
        title: form.title.trim(),
        reminderType: form.reminderType,
        taskId: null,
        scheduleTime: form.scheduleTime,
        daysOfWeek: form.days,
        specificDate: form.reminderType === "specific_date" ? form.specificDate || null : null,
      };
      if (form.id) {
        await api.updateReminder({ ...payload, id: form.id, enabled: form.enabled });
        setToastMessage("Reminder updated and scheduled task synced.");
      } else {
        await api.createReminder(payload);
        setToastMessage("Reminder created and Windows scheduled task registered.");
      }
      setForm(EMPTY_FORM);
      setEditing(false);
      await load();
    } catch (caught) {
      setToastMessage(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setSaving(false);
    }
  };

  const remove = async (id: string) => {
    if (!window.confirm("Delete this reminder and its scheduled task?")) return;
    try {
      await api.deleteReminder(id);
      setToastMessage("Reminder deleted.");
      await load();
    } catch (caught) {
      setToastMessage(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const toggleEnabled = async (r: ReminderConfig) => {
    try {
      await api.setReminderEnabled(r.id, !r.enabled);
      await load();
    } catch (caught) {
      setToastMessage(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const snooze = async (id: string, minutes: number) => {
    try {
      await api.snoozeReminder(id, minutes);
      setToastMessage(`Snoozed ${minutes} minutes.`);
      await load();
    } catch (caught) {
      setToastMessage(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const today = bootstrap?.today ?? "";

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Schedule"
        title="Calendar"
        description="Daily and weekly production reminders via Windows Task Scheduler. Quiet hours suppress delivery and are logged — never shown."
        actions={<Badge variant="success">Milestone 5 — live</Badge>}
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
              <CardDescription>Each enabled reminder = one Windows task `StoryQuestStudio\…`.</CardDescription>
            </div>
            <div className="flex gap-2">
              <Button variant="secondary" size="sm" onClick={() => { setForm(EMPTY_FORM); setEditing(true); }}>
                <Plus className="h-3.5 w-3.5" /> New
              </Button>
              <Button variant="secondary" size="sm" onClick={() => void load()} disabled={loading}>
                {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Clock className="h-3.5 w-3.5" />}
                Refresh
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            {error ? <p className="text-sm text-red-300">{error}</p> : null}
            {editing && (
              <div className="space-y-3 rounded-xl border border-white/[0.08] bg-obsidian-900/60 p-4">
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium text-slate-200">{form.id ? "Edit reminder" : "New reminder"}</p>
                  <Button variant="ghost" size="sm" onClick={() => { setEditing(false); setForm(EMPTY_FORM); }}>
                    <X className="h-4 w-4" />
                  </Button>
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="rem-title">Title</Label>
                  <Input id="rem-title" value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />
                </div>
                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1.5">
                    <Label htmlFor="rem-type">Type</Label>
                    <select
                      id="rem-type"
                      className="w-full rounded-md border border-white/10 bg-obsidian-900 px-2 py-2 text-sm text-slate-200"
                      value={form.reminderType}
                      onChange={(e) => setForm({ ...form, reminderType: e.target.value })}
                    >
                      {REMINDER_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
                    </select>
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="rem-time">Time (HH:MM)</Label>
                    <Input id="rem-time" type="time" value={form.scheduleTime}
                      onChange={(e) => setForm({ ...form, scheduleTime: e.target.value })} />
                  </div>
                </div>
                {(form.reminderType === "weekly" || form.reminderType === "weekly_review") && (
                  <div className="flex flex-wrap gap-1.5">
                    {DAY_LABELS.map((label, day) => (
                      <button
                        key={label}
                        type="button"
                        onClick={() => toggleDay(day)}
                        className={`rounded-md border px-2 py-1 text-xs ${form.days.includes(day)
                          ? "border-amber-400/40 bg-amber-400/10 text-amber-200"
                          : "border-white/10 text-slate-500"}`}
                      >
                        {label}
                      </button>
                    ))}
                  </div>
                )}
                {form.reminderType === "specific_date" && (
                  <div className="space-y-1.5">
                    <Label htmlFor="rem-date">Date</Label>
                    <Input id="rem-date" type="date" value={form.specificDate}
                      onChange={(e) => setForm({ ...form, specificDate: e.target.value })} />
                  </div>
                )}
                <label className="flex items-center gap-2 text-sm text-slate-300">
                  <input type="checkbox" checked={form.enabled}
                    onChange={(e) => setForm({ ...form, enabled: e.target.checked })} />
                  Enabled — register Windows scheduled task
                </label>
                <Button disabled={saving || !form.title.trim()} onClick={() => void save()}>
                  {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : null} Save reminder
                </Button>
              </div>
            )}
            {!loading && reminders.length === 0 && !error && !editing ? (
              <EmptyState
                icon={Bell}
                title="No reminders configured"
                description="Click New to create a daily/weekly reminder. Saving registers a Windows scheduled task that fires even when the app is closed."
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
                        {reminder.reminderType} · task `{reminder.windowsTaskName}` · last fired {reminder.lastTriggeredAt ?? "never"}
                        {reminder.snoozedUntil ? ` · snoozed until ${reminder.snoozedUntil}` : ""}
                      </p>
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Badge variant={reminder.enabled ? "success" : "muted"}>
                      {reminder.enabled ? "Enabled" : "Disabled"}
                    </Badge>
                    <Button variant="ghost" size="sm" onClick={() => {
                      setForm({
                        id: reminder.id,
                        title: reminder.title,
                        reminderType: reminder.reminderType,
                        scheduleTime: reminder.scheduleTime,
                        days: parseDaysArray(reminder.daysOfWeekJson),
                        specificDate: reminder.specificDate ?? "",
                        enabled: reminder.enabled,
                      });
                      setEditing(true);
                    }}>
                      <Pencil className="h-3.5 w-3.5" />
                    </Button>
                    <Button variant="ghost" size="sm" onClick={() => void toggleEnabled(reminder)}>
                      {reminder.enabled ? "Disable" : "Enable"}
                    </Button>
                    <Button variant="ghost" size="sm" onClick={() => void snooze(reminder.id, 30)}>Snooze 30m</Button>
                    <Button variant="ghost" size="sm" onClick={() => void remove(reminder.id)}>
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              ))
            )}
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Test notification</CardTitle>
              <CardDescription>Respects quiet hours — suppression is logged, not shown.</CardDescription>
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
                {reminders.filter((r) => r.enabled).length} enabled reminder(s). Scheduled tasks run
                `--reminder-mode --reminder-id …` even when the app is closed; missed runs are recovered
                on next launch as a single summary.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
