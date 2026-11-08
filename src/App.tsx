import { listen } from "@tauri-apps/api/event";
import { MonitorX } from "lucide-react";
import { useEffect } from "react";

import { ErrorState } from "./components/common/error-state";
import { LoadingScreen } from "./components/common/loading-screen";
import { NoticeToast } from "./components/common/notice-toast";
import { AppShell } from "./components/layout/app-shell";
import { Button } from "./components/ui/button";
import { Card, CardContent } from "./components/ui/card";
import { isTauriRuntime } from "./lib/tauriBridge";
import { screenTitle } from "./lib/navigation";
import { useStoryQuestStore } from "./store/useStoryQuestStore";
import type { ScreenId } from "./types/storyquest";
import { AchievementsScreen } from "./screens/achievements-screen";
import { AiPlannerScreen } from "./screens/ai-planner-screen";
import { AssetsScreen } from "./screens/assets-screen";
import { CalendarScreen } from "./screens/calendar-screen";
import { CampaignsScreen } from "./screens/campaigns-screen";
import { DailyQuestsScreen } from "./screens/daily-quests-screen";
import { DashboardScreen } from "./screens/dashboard-screen";
import { JourneyScreen } from "./screens/journey-screen";
import { QuestMapScreen } from "./screens/quest-map-screen";
import { SettingsScreen } from "./screens/settings-screen";
import { YouTubeScreen } from "./screens/youtube-screen";

const SCREEN_IDS: ScreenId[] = [
  "dashboard",
  "journey",
  "campaigns",
  "quest-map",
  "daily-quests",
  "youtube",
  "achievements",
  "assets",
  "calendar",
  "ai-planner",
  "settings",
];

function renderScreen(screen: ScreenId) {
  switch (screen) {
    case "dashboard":
      return <DashboardScreen />;
    case "journey":
      return <JourneyScreen />;
    case "campaigns":
      return <CampaignsScreen />;
    case "quest-map":
      return <QuestMapScreen />;
    case "daily-quests":
      return <DailyQuestsScreen />;
    case "youtube":
      return <YouTubeScreen />;
    case "achievements":
      return <AchievementsScreen />;
    case "assets":
      return <AssetsScreen />;
    case "calendar":
      return <CalendarScreen />;
    case "ai-planner":
      return <AiPlannerScreen />;
    case "settings":
      return <SettingsScreen />;
    default:
      return <DashboardScreen />;
  }
}

function DesktopRequired() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-obsidian-950 p-6 text-slate-100">
      <Card className="max-w-xl">
        <CardContent className="flex flex-col items-center p-8 text-center">
          <div className="mb-5 rounded-2xl border border-amber-400/20 bg-amber-400/[0.06] p-3 text-amber-300">
            <MonitorX className="h-7 w-7" />
          </div>
          <h1 className="font-display text-2xl font-semibold">Desktop runtime required</h1>
          <p className="mt-3 text-sm leading-relaxed text-slate-400">
            This is the StoryQuest Studio interface running in a browser preview. The real app
            needs its Tauri desktop runtime for SQLite, backups, Windows startup, and
            notifications.
          </p>
          <p className="mt-4 rounded-xl border border-white/[0.06] bg-obsidian-900/70 px-4 py-2 font-mono text-xs text-slate-400">
            npm run tauri:dev
          </p>
          <p className="mt-3 text-xs text-slate-600">
            Or launch the installed Windows build. No sample statistics are shown here.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

export default function App() {
  const { activeScreen, bootstrap, loading, error, notice, loadBootstrap, dismissNotice, setScreen } =
    useStoryQuestStore();

  useEffect(() => {
    void loadBootstrap();
  }, [loadBootstrap]);

  useEffect(() => {
    document.title = `${screenTitle(activeScreen)} — StoryQuest Studio`;
  }, [activeScreen]);

  useEffect(() => {
    if (!isTauriRuntime()) return undefined;
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listen<string>("tray://navigate", (event) => {
      if (SCREEN_IDS.includes(event.payload as ScreenId)) {
        setScreen(event.payload as ScreenId);
      }
    }).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });

    return () => {
      disposed = true;
      if (unlisten) unlisten();
    };
  }, [setScreen]);

  if (!isTauriRuntime()) {
    return <DesktopRequired />;
  }

  if (loading && !bootstrap) {
    return (
      <div className="h-screen bg-obsidian-950 text-slate-100">
        <LoadingScreen />
      </div>
    );
  }

  if (error && !bootstrap) {
    return (
      <div className="h-screen bg-obsidian-950 text-slate-100">
        <ErrorState message={error} onRetry={() => void loadBootstrap()} />
      </div>
    );
  }

  if (!bootstrap) {
    return (
      <div className="h-screen bg-obsidian-950 text-slate-100">
        <LoadingScreen />
      </div>
    );
  }

  return (
    <>
      <AppShell>
        {error ? (
          <Card className="mb-4 border-red-400/20">
            <CardContent className="flex items-center justify-between gap-3 p-3 text-sm text-red-200">
              <span>{error}</span>
              <Button size="sm" variant="secondary" onClick={() => void loadBootstrap()}>
                Retry
              </Button>
            </CardContent>
          </Card>
        ) : null}
        {renderScreen(activeScreen)}
      </AppShell>
      <NoticeToast notice={notice} onDismiss={dismissNotice} />
    </>
  );
}
