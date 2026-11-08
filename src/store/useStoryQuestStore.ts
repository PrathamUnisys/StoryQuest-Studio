import { create } from "zustand";

import { api } from "../lib/commands";
import { isTauriRuntime } from "../lib/tauriBridge";
import type {
  AcknowledgeNotificationsInput,
  AchievementNotification,
  ArchiveCampaignInput,
  BootstrapState,
  CreateCampaignInput,
  CreateEpisodeInput,
  ReopenTaskInput,
  ScreenId,
  SetStreakPrefsInput,
  SetTaskStatusInput,
  ThresholdUpdate,
  UpdateCampaignInput,
  UpdateEpisodeInput,
  UserProfile,
} from "../types/storyquest";

interface Notice {
  kind: "success" | "error";
  message: string;
}

interface StoryQuestState {
  activeScreen: ScreenId;
  bootstrap: BootstrapState | null;
  loading: boolean;
  saving: boolean;
  error: string | null;
  notice: Notice | null;
  notifications: AchievementNotification[];
  setScreen: (screen: ScreenId) => void;
  loadBootstrap: () => Promise<void>;
  updateProfile: (displayName: string, studioName: string) => Promise<void>;
  saveSetting: (key: string, value: string) => Promise<void>;
  saveSettings: (items: { key: string; value: string }[]) => Promise<void>;
  updateThresholds: (thresholds: ThresholdUpdate[]) => Promise<void>;
  recalculateRank: () => Promise<void>;
  completeOnboarding: () => Promise<void>;
  backupDatabase: () => Promise<void>;
  dismissNotice: () => void;
  createCampaign: (input: CreateCampaignInput) => Promise<void>;
  updateCampaign: (input: UpdateCampaignInput) => Promise<void>;
  archiveCampaign: (input: ArchiveCampaignInput) => Promise<void>;
  createEpisode: (input: CreateEpisodeInput) => Promise<void>;
  updateEpisode: (input: UpdateEpisodeInput) => Promise<void>;
  setTaskStatus: (input: SetTaskStatusInput) => Promise<void>;
  reopenTask: (input: ReopenTaskInput) => Promise<void>;
  fetchNotifications: () => Promise<void>;
  acknowledgeNotifications: (input: AcknowledgeNotificationsInput) => Promise<void>;
  setStreakPrefs: (input: SetStreakPrefsInput) => Promise<void>;
  youtubeSetClientCredentials: (clientId: string, clientSecret: string) => Promise<void>;
  youtubeConnect: (loginHint?: string) => Promise<void>;
  youtubeSyncNow: () => Promise<void>;
  youtubeDisconnect: () => Promise<void>;
}

function messageFrom(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "Something went wrong.";
}

export const useStoryQuestStore = create<StoryQuestState>((set, get) => ({
  activeScreen: "dashboard",
  bootstrap: null,
  loading: true,
  saving: false,
  error: null,
  notice: null,
  notifications: [],

  setScreen: (screen) => set({ activeScreen: screen }),

  loadBootstrap: async () => {
    if (!isTauriRuntime()) {
      set({
        loading: false,
        error:
          "StoryQuest Studio needs its desktop runtime. Launch with `npm run tauri:dev` or the installed Windows app.",
      });
      return;
    }

    set({ loading: true, error: null });
    try {
      const bootstrap = await api.getBootstrap();
      set({ bootstrap, loading: false });
      await get().fetchNotifications();
    } catch (error) {
      set({ loading: false, error: messageFrom(error) });
    }
  },

  updateProfile: async (displayName, studioName) => {
    set({ saving: true, notice: null });
    try {
      await api.updateProfile(displayName, studioName);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Profile saved." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  saveSetting: async (key, value) => {
    set({ saving: true, notice: null });
    try {
      await api.setSetting(key, value);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Setting saved." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  saveSettings: async (items) => {
    set({ saving: true, notice: null });
    try {
      for (const item of items) {
        await api.setSetting(item.key, item.value);
      }
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Settings saved." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  updateThresholds: async (thresholds) => {
    set({ saving: true, notice: null });
    try {
      await api.updateCreatorThresholds(thresholds);
      await get().loadBootstrap();
      set({
        saving: false,
        notice: { kind: "success", message: "Creator rank thresholds saved." },
      });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  recalculateRank: async () => {
    set({ saving: true, notice: null });
    try {
      await api.recalculateCreatorRank();
      await get().loadBootstrap();
      set({
        saving: false,
        notice: { kind: "success", message: "Creator rank recalculated." },
      });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  completeOnboarding: async () => {
    set({ saving: true, notice: null });
    try {
      await api.completeOnboarding();
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Onboarding complete." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  backupDatabase: async () => {
    set({ saving: true, notice: null });
    try {
      const path = await api.backupDatabase();
      set({
        saving: false,
        notice: { kind: "success", message: `Backup created: ${path}` },
      });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  createCampaign: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.createCampaign(input);
      await get().loadBootstrap();
      set({
        saving: false,
        notice: { kind: "success", message: `Campaign "${input.title}" created.` },
      });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  updateCampaign: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.updateCampaign(input);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Campaign updated." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  archiveCampaign: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.archiveCampaign(input);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Campaign archived." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  createEpisode: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.createEpisode(input);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: `Episode "${input.title}" created.` } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  updateEpisode: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.updateEpisode(input);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Episode updated." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  setTaskStatus: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.setTaskStatus(input);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: `Task set to ${input.status}.` } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  reopenTask: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.reopenTask(input);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Task reopened." } });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  dismissNotice: () => set({ notice: null }),

  fetchNotifications: async () => {
    set({ saving: true, notice: null });
    try {
      const notifications = await api.getAchievementNotifications();
      set({ saving: false, notifications });
    } catch (error) {
      set({
        saving: false,
        notifications: [],
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  acknowledgeNotifications: async (input) => {
    set({ saving: true, notice: null });
    try {
      const ok = await api.acknowledgeAchievementNotification(input);
      if (ok) {
        set((prev) => ({
          saving: false,
          notifications: prev.notifications.filter(
            (n) => !input.notificationIds.includes(n.id)
          ),
        }));
      } else {
        set({ saving: false });
      }
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  setStreakPrefs: async (input) => {
    set({ saving: true, notice: null });
    try {
      await api.setStreakPrefs(input);
      await get().loadBootstrap();
      await get().fetchNotifications();
      set({
        saving: false,
        notice: { kind: "success", message: "Streak preferences saved." },
      });
    } catch (error) {
      set({
        saving: false,
        notice: { kind: "error", message: messageFrom(error) },
      });
    }
  },

  youtubeSetClientCredentials: async (clientId, clientSecret) => {
    set({ saving: true, notice: null });
    try {
      await api.youtubeSetClientCredentials({ clientId, clientSecret });
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "Google OAuth credentials saved." } });
    } catch (error) {
      set({ saving: false, notice: { kind: "error", message: messageFrom(error) } });
    }
  },

  youtubeConnect: async (loginHint) => {
    set({ saving: true, notice: null });
    try {
      await api.youtubeConnect(loginHint);
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "YouTube connected. Run Sync now to fetch channel stats." } });
    } catch (error) {
      set({ saving: false, notice: { kind: "error", message: messageFrom(error) } });
    }
  },

  youtubeSyncNow: async () => {
    set({ saving: true, notice: null });
    try {
      await api.youtubeSyncNow();
      await get().loadBootstrap();
      await get().fetchNotifications();
      set({ saving: false, notice: { kind: "success", message: "YouTube channel synced." } });
    } catch (error) {
      set({ saving: false, notice: { kind: "error", message: messageFrom(error) } });
    }
  },

  youtubeDisconnect: async () => {
    set({ saving: true, notice: null });
    try {
      await api.youtubeDisconnect();
      await get().loadBootstrap();
      set({ saving: false, notice: { kind: "success", message: "YouTube disconnected." } });
    } catch (error) {
      set({ saving: false, notice: { kind: "error", message: messageFrom(error) } });
    }
  },
}));
