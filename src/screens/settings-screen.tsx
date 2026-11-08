import {
  Award,
  Calendar,
  Check,
  Database,
  FolderOpen,
  Info,
  Loader2,
  Save,
  Settings2,
  ShieldCheck,
  UserRound,
} from "lucide-react";
import { useEffect, useState } from "react";

import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Select, Switch } from "../components/ui/select";
import { Separator } from "../components/ui/separator";
import { api } from "../lib/commands";
import { getBoolSetting, getSetting } from "../lib/format";
import { useStoryQuestStore } from "../store/useStoryQuestStore";
import type { ThresholdUpdate } from "../types/storyquest";

type Tab = "profile" | "ranks" | "startup" | "data" | "about";

const TABS: { id: Tab; label: string; icon: typeof UserRound }[] = [
  { id: "profile", label: "Profile", icon: UserRound },
  { id: "ranks", label: "Creator ranks", icon: Award },
  { id: "startup", label: "Startup & reminders", icon: Settings2 },
  { id: "data", label: "Data & backup", icon: Database },
  { id: "about", label: "About", icon: Info },
];

export function SettingsScreen() {
  const { bootstrap, saving, updateProfile, saveSetting, saveSettings, updateThresholds, backupDatabase } =
    useStoryQuestStore();
  const [tab, setTab] = useState<Tab>("profile");

  const [displayName, setDisplayName] = useState("");
  const [studioName, setStudioName] = useState("");
  const [thresholds, setThresholds] = useState<ThresholdUpdate[]>([]);
  const [thresholdError, setThresholdError] = useState<string | null>(null);
  const [quietStart, setQuietStart] = useState("22:30");
  const [quietEnd, setQuietEnd] = useState("07:30");
  const [quietEnabled, setQuietEnabled] = useState(true);
  const [startupError, setStartupError] = useState<string | null>(null);
  const [restDay, setRestDay] = useState(0);
  const [vacationMode, setVacationMode] = useState(false);
  const [vacationUntil, setVacationUntil] = useState("");

  useEffect(() => {
    if (!bootstrap) return;
    setDisplayName(bootstrap.profile.displayName);
    setStudioName(bootstrap.profile.studioName);
    setThresholds(
      bootstrap.creatorRanks.map((rank) => ({
        level: rank.level,
        minPublishedVideos: rank.minPublishedVideos,
      }))
    );
    setQuietStart(getSetting(bootstrap.settings, "quiet_hours_start", "22:30"));
    setQuietEnd(getSetting(bootstrap.settings, "quiet_hours_end", "07:30"));
    setQuietEnabled(getBoolSetting(bootstrap.settings, "quiet_hours_enabled", true));
    setRestDay(bootstrap.streak.weeklyRestDay);
    setVacationMode(bootstrap.streak.vacationMode);
    setVacationUntil(bootstrap.streak.vacationUntil ?? "");
  }, [bootstrap]);

  if (!bootstrap) return null;

  const startWithWindows = getBoolSetting(bootstrap.settings, "start_with_windows", false);
  const startMinimized = getBoolSetting(bootstrap.settings, "start_minimized", false);
  const minimizeToTray = getBoolSetting(bootstrap.settings, "minimize_to_tray", true);
  const youtubeSync = getBoolSetting(bootstrap.settings, "youtube_sync_enabled", false);

  const saveRanks = () => {
    setThresholdError(null);
    const sorted = [...thresholds].sort((a, b) => a.level - b.level);
    if (sorted[0]?.minPublishedVideos !== 0) {
      setThresholdError("Rank 1 must start at 0 published videos.");
      return;
    }
    for (let index = 1; index < sorted.length; index += 1) {
      if (sorted[index].minPublishedVideos <= sorted[index - 1].minPublishedVideos) {
        setThresholdError("Published-video thresholds must increase with each rank.");
        return;
      }
    }
    void updateThresholds(sorted);
  };

  const toggleStartup = async (key: string, value: boolean) => {
    setStartupError(null);
    await saveSetting(key, value ? "true" : "false");
    if (key === "start_with_windows") {
      try {
        await api.applyAutostart(value);
      } catch (caught) {
        setStartupError(caught instanceof Error ? caught.message : String(caught));
      }
    }
  };

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Studio"
        title="Settings"
        description="Profile, creator ranks, Windows startup, reminders, and local data. Only settings backed by real commands are editable."
      />

      <div className="flex flex-wrap gap-2">
        {TABS.map((item) => {
          const Icon = item.icon;
          const active = tab === item.id;
          return (
            <Button
              key={item.id}
              variant={active ? "default" : "secondary"}
              size="sm"
              onClick={() => setTab(item.id)}
            >
              <Icon className="h-3.5 w-3.5" />
              {item.label}
            </Button>
          );
        })}
      </div>

      {tab === "profile" ? (
        <div className="grid gap-6 xl:grid-cols-[1.2fr_1fr]">
          <Card>
            <CardHeader>
              <CardTitle>Creator profile</CardTitle>
              <CardDescription>
                Stored locally in SQLite. The original demo identity and fake progression have
                been cleared.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-5">
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="displayName">Creator name</Label>
                  <Input
                    id="displayName"
                    value={displayName}
                    onChange={(event) => setDisplayName(event.target.value)}
                    placeholder="Your name"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="studioName">Studio name</Label>
                  <Input
                    id="studioName"
                    value={studioName}
                    onChange={(event) => setStudioName(event.target.value)}
                    placeholder="Your studio"
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label>Profile identifier</Label>
                <Input value={bootstrap.profile.id} readOnly className="text-slate-500" />
              </div>
              <div className="flex flex-wrap gap-2">
                <Button
                  onClick={() => void updateProfile(displayName, studioName)}
                  disabled={saving}
                >
                  {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
                  Save profile
                </Button>
                {!bootstrap.onboardingCompleted ? (
                  <Button
                    variant="secondary"
                    disabled={saving}
                    onClick={() => void useStoryQuestStore.getState().completeOnboarding()}
                  >
                    <Check className="h-4 w-4" />
                    Complete onboarding
                  </Button>
                ) : (
                  <Badge variant="success">Onboarding complete</Badge>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Account state</CardTitle>
              <CardDescription>What the database currently records.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <Row label="XP level" value={`${bootstrap.profile.currentLevel} · ${bootstrap.profile.activeTitle}`} />
              <Row label="Total XP" value={`${bootstrap.profile.totalXp}`} />
              <Row label="Creator rank" value={bootstrap.creatorRank.rankName} />
              <Row label="Published videos" value={`${bootstrap.creatorProgress.publishedVideosTotal}`} />
              <Row label="Current streak" value={`${bootstrap.streak.currentStreak} days`} />
              <Row label="YouTube" value={bootstrap.youtube.connected ? "Connected" : "Not connected"} />
              <p className="pt-2 text-xs leading-relaxed text-slate-600">
                Creator rank uses published videos. XP level is separate and only changes when a
                real XP award command runs.
              </p>
            </CardContent>
          </Card>
        </div>
      ) : null}

      {tab === "ranks" ? (
        <Card>
          <CardHeader>
            <CardTitle>Creator rank thresholds</CardTitle>
            <CardDescription>
              Edit the published-video count required for each rank. They must start at 0 and
              increase strictly. Saving immediately recalculates your current rank.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              {thresholds.map((threshold, index) => {
                const rank = bootstrap.creatorRanks[index];
                return (
                  <div
                    key={threshold.level}
                    className="grid items-center gap-3 rounded-xl border border-white/[0.06] bg-obsidian-900/60 px-3 py-2 sm:grid-cols-[50px_1fr_120px]"
                  >
                    <span className="text-sm font-semibold text-amber-300">
                      {threshold.level}
                    </span>
                    <span className="truncate text-sm text-slate-300">
                      {rank?.rankName ?? `Rank ${threshold.level}`}
                    </span>
                    <div className="flex items-center gap-2">
                      <Input
                        type="number"
                        min={threshold.level === 1 ? 0 : 1}
                        value={threshold.minPublishedVideos}
                        onChange={(event) => {
                          const value = Number(event.target.value);
                          setThresholds((current) =>
                            current.map((item) =>
                              item.level === threshold.level
                                ? { ...item, minPublishedVideos: Number.isFinite(value) ? value : 0 }
                                : item
                            )
                          );
                        }}
                        disabled={threshold.level === 1}
                        className="h-9 text-right"
                      />
                      <span className="text-xs text-slate-600">videos</span>
                    </div>
                  </div>
                );
              })}
            </div>

            {thresholdError ? <p className="text-sm text-red-300">{thresholdError}</p> : null}

            <div className="flex flex-wrap gap-2">
              <Button onClick={saveRanks} disabled={saving}>
                {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
                Save thresholds
              </Button>
              <Button
                variant="secondary"
                disabled={saving}
                onClick={() => void useStoryQuestStore.getState().recalculateRank()}
              >
                Recalculate rank
              </Button>
            </div>

            <Separator />
            <p className="text-xs leading-relaxed text-slate-600">
              Current: {bootstrap.creatorRank.rankName} at{" "}
              {bootstrap.creatorProgress.publishedVideosTotal} published videos.
            </p>
          </CardContent>
        </Card>
      ) : null}

      {tab === "startup" ? (
        <div className="grid gap-6 xl:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Windows startup</CardTitle>
              <CardDescription>
                Uses the Tauri autostart plugin. Enabling startup registers the installed
                executable with Windows.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-5">
              <ToggleRow
                label="Start with Windows"
                description="Register StoryQuest Studio to launch at sign-in."
                checked={startWithWindows}
                onChange={(value) => void toggleStartup("start_with_windows", value)}
              />
              <ToggleRow
                label="Start minimized"
                description="Open in the background with the tray icon."
                checked={startMinimized}
                onChange={(value) => void toggleStartup("start_minimized", value)}
              />
              <ToggleRow
                label="Close to tray"
                description="Closing the window keeps reminders available."
                checked={minimizeToTray}
                onChange={(value) => void toggleStartup("minimize_to_tray", value)}
              />
              {startupError ? <p className="text-sm text-red-300">{startupError}</p> : null}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Quiet hours</CardTitle>
              <CardDescription>
                Active: reminders fired inside this window are suppressed and logged to notification history.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <ToggleRow
                label="Enable quiet hours"
                description="Suppress reminders during the window."
                checked={quietEnabled}
                onChange={setQuietEnabled}
              />
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="quietStart">Start</Label>
                  <Input
                    id="quietStart"
                    type="time"
                    value={quietStart}
                    onChange={(event) => setQuietStart(event.target.value)}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="quietEnd">End</Label>
                  <Input
                    id="quietEnd"
                    type="time"
                    value={quietEnd}
                    onChange={(event) => setQuietEnd(event.target.value)}
                  />
                </div>
              </div>
              <Button
                variant="secondary"
                disabled={saving}
                onClick={() =>
                  void saveSettings([
                    { key: "quiet_hours_enabled", value: quietEnabled ? "true" : "false" },
                    { key: "quiet_hours_start", value: quietStart },
                    { key: "quiet_hours_end", value: quietEnd },
                  ])
                }
              >
                <Save className="h-4 w-4" />
                Save quiet hours
              </Button>
            </CardContent>
          </Card>

          <Card className="xl:col-span-2">
            <CardHeader>
              <CardTitle>YouTube sync</CardTitle>
              <CardDescription>
                Disabled until a channel is connected through the official OAuth flow in
                Milestone 4.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ToggleRow
                label="Scheduled channel sync"
                description={
                  youtubeSync
                    ? "Currently enabled."
                    : "Not available until a channel connection exists."
                }
                checked={youtubeSync}
                disabled
                onChange={() => undefined}
              />
            </CardContent>
          </Card>

          <Card className="xl:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Calendar className="h-4 w-4 text-amber-300" />
                Streak preferences
              </CardTitle>
              <CardDescription>
                Rest days never count as missed days. Vacation days protect your streak for the
                configured window. Freezes bridge eligible missed days.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>Rest day</Label>
                <select
                  value={String(restDay)}
                  onChange={(event) => setRestDay(Number(event.target.value))}
                  className="rounded-xl border border-white/10 bg-obsidian-900/80 px-3 py-2 text-sm text-slate-100"
                >
                  {[
                    "Sunday",
                    "Monday",
                    "Tuesday",
                    "Wednesday",
                    "Thursday",
                    "Friday",
                    "Saturday",
                  ].map((day, index) => (
                    <option key={index} value={String(index)}>
                      {day}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Switch
                    checked={vacationMode}
                    onChange={() => setVacationMode(!vacationMode)}
                    aria-label="Vacation mode"
                  />
                  <span className="text-sm text-slate-300">
                    {vacationMode ? "Vacation mode active" : "Vacation mode inactive"}
                  </span>
                </div>
                {vacationMode && (
                  <div className="space-y-2">
                    <Label>Vacation until (inclusive)</Label>
                    <Input
                      type="date"
                      value={vacationUntil}
                      onChange={(event) => setVacationUntil(event.target.value)}
                    />
                  </div>
                )}
              </div>
              <Button
                onClick={() =>
                  void useStoryQuestStore.getState().setStreakPrefs({
                    weeklyRestDay: restDay,
                    vacationMode,
                    vacationUntil: vacationUntil || null,
                  })
                }
                disabled={saving}
              >
                {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Save className="h-4 w-4" />}
                Save streak preferences
              </Button>
            </CardContent>
          </Card>
        </div>
      ) : null}

      {tab === "data" ? (
        <div className="grid gap-6 xl:grid-cols-[1.2fr_1fr]">
          <Card>
            <CardHeader>
              <CardTitle>Local data</CardTitle>
              <CardDescription>
                Everything is stored on this Windows machine. No cloud sync is active.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <PathRow label="Data folder" value={bootstrap.appInfo.dataDir} />
              <PathRow label="Database" value={bootstrap.appInfo.dbPath} />
              <Row label="Schema version" value={bootstrap.appInfo.schemaVersion} />
              <div className="flex flex-wrap gap-2 pt-1">
                <Button variant="secondary" onClick={() => void api.openDataFolder()}>
                  <FolderOpen className="h-4 w-4" />
                  Open data folder
                </Button>
                <Button onClick={() => void backupDatabase()} disabled={saving}>
                  {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Database className="h-4 w-4" />}
                  Back up database
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Data origin</CardTitle>
              <CardDescription>What is real, what is missing.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-slate-400">
              <LegendRow color="emerald" label="Local database" detail="Profile, ranks, tasks, settings" />
              <LegendRow color="amber" label="Sample content" detail="Starter campaign, reset to not started" />
              <LegendRow color="muted" label="Unavailable" detail="YouTube metrics until Milestone 4" />
              <LegendRow color="muted" label="Milestone 2+" detail="Daily quests, assets, campaign editing" />
              <p className="pt-2 text-xs leading-relaxed text-slate-600">
                Backups use SQLite `VACUUM INTO` and are written to the local Backups folder.
                Pre-migration backups are created automatically before schema changes.
              </p>
            </CardContent>
          </Card>
        </div>
      ) : null}

      {tab === "about" ? (
        <div className="grid gap-6 xl:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>StoryQuest Studio</CardTitle>
              <CardDescription>{bootstrap.appInfo.name}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <Row label="Version" value={bootstrap.appInfo.version} />
              <Row label="Identifier" value={bootstrap.appInfo.identifier} />
              <Row label="Schema" value={bootstrap.appInfo.schemaVersion} />
              <Row label="Platform" value="Windows · Tauri 2 · React · SQLite" />
              <div className="flex items-center gap-2 pt-2 text-xs text-emerald-300/90">
                <ShieldCheck className="h-4 w-4" />
                Local-first. No hidden network calls in Milestone 1.
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Deliberate limitations</CardTitle>
              <CardDescription>Honest status for this milestone.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-slate-400">
              <Limitation text="Daily quest generator, focus timer, and XP completion flow — Milestone 2." />
              <Limitation text="Campaign and episode creation, asset library, roadmap editing — Milestone 2." />
              <Limitation text="YouTube OAuth, channel analytics, publications — Milestone 4." />
              <Limitation text="AI provider settings and roadmap generation — Milestone 6." />
              <Limitation text="AI planner, MCP dispatch, and skill tree remain in the codebase but are not in the main navigation yet." />
            </CardContent>
          </Card>
        </div>
      ) : null}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-slate-500">{label}</span>
      <span className="text-right font-medium text-slate-300">{value}</span>
    </div>
  );
}

function PathRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="space-y-1.5">
      <p className="text-xs font-semibold uppercase tracking-wider text-slate-500">{label}</p>
      <p className="break-all rounded-xl border border-white/[0.06] bg-obsidian-900/60 px-3 py-2 font-mono text-xs text-slate-400">
        {value}
      </p>
    </div>
  );
}

function ToggleRow({
  label,
  description,
  checked,
  onChange,
  disabled = false,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div>
        <p className="text-sm font-medium text-slate-300">{label}</p>
        <p className="mt-0.5 text-xs leading-relaxed text-slate-600">{description}</p>
      </div>
      <Switch
        checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        aria-label={label}
      />
    </div>
  );
}

function LegendRow({
  color,
  label,
  detail,
}: {
  color: "emerald" | "amber" | "muted";
  label: string;
  detail: string;
}) {
  const dot = {
    emerald: "bg-emerald-400",
    amber: "bg-amber-400",
    muted: "bg-slate-600",
  }[color];
  return (
    <div className="flex items-start gap-2.5">
      <span className={`mt-1.5 h-2 w-2 shrink-0 rounded-full ${dot}`} />
      <div>
        <p className="text-sm text-slate-300">{label}</p>
        <p className="text-xs text-slate-600">{detail}</p>
      </div>
    </div>
  );
}

function Limitation({ text }: { text: string }) {
  return (
    <div className="flex items-start gap-2">
      <span className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-slate-600" />
      <p className="text-sm leading-relaxed">{text}</p>
    </div>
  );
}
