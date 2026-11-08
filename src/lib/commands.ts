import { invoke } from "@tauri-apps/api/core";

import type {
  AcknowledgeNotificationsInput,
  AchievementNotification,
  ActivityDay,
  AppInfo,
  ArchiveCampaignInput,
  BootstrapState,
  Campaign,
  CreateCampaignInput,
  CreateEpisodeInput,
  CreatorProgress,
  CreatorRank,
  Episode,
  QuestTask,
  ReminderConfig,
  ReopenTaskInput,
  SetStreakPrefsInput,
  SetTaskStatusInput,
  SettingItem,
  StreakRecord,
  ThresholdUpdate,
  UpdateCampaignInput,
  UpdateEpisodeInput,
  UserProfile,
  CreateReminderInput,
  UpdateReminderInput,
  YoutubeClientCredentialsInput,
  YoutubeStatus,
} from "../types/storyquest";
import { isTauriRuntime } from "./tauriBridge";

export class DesktopRequiredError extends Error {
  constructor() {
    super(
      "StoryQuest Studio needs its desktop runtime. Launch the app with `npm run tauri:dev` or the installed Windows build."
    );
    this.name = "DesktopRequiredError";
  }
}

function assertDesktop() {
  if (!isTauriRuntime()) {
    throw new DesktopRequiredError();
  }
}

async function call<T>(command: string, args?: unknown): Promise<T> {
  assertDesktop();
  return invoke<T>(command, args as Record<string, unknown>);
}

export const api = {
  getBootstrap: () => call<BootstrapState>("cmd_get_bootstrap"),
  updateProfile: (displayName: string, studioName: string, avatarBadge?: string) =>
    call<UserProfile>("cmd_update_profile", { displayName, studioName, avatarBadge }),
  completeOnboarding: () => call<boolean>("cmd_complete_onboarding"),
  getSettings: () => call<SettingItem[]>("cmd_get_settings"),
  setSetting: (key: string, value: string) =>
    call<SettingItem>("cmd_set_setting", { key, value }),
  getCreatorRanks: () => call<CreatorRank[]>("cmd_get_creator_ranks"),
  updateCreatorThresholds: (thresholds: ThresholdUpdate[]) =>
    call<CreatorProgress>("cmd_update_creator_thresholds", { thresholds }),
  recalculateCreatorRank: () => call<CreatorProgress>("cmd_recalculate_creator_rank"),
  getAppInfo: () => call<AppInfo>("cmd_get_app_info"),
  openDataFolder: () => call<void>("cmd_open_data_folder"),
  listCampaigns: () => call<Campaign[]>("cmd_list_campaigns"),
  listEpisodes: (campaignId: string) =>
    call<Episode[]>("cmd_list_episodes", { campaignId }),
  listEpisodeTasks: (episodeId: string) =>
    call<QuestTask[]>("cmd_list_episode_tasks", { episodeId }),
  listReminders: () => call<ReminderConfig[]>("cmd_list_reminders"),
  applyAutostart: (enabled: boolean) =>
    call<boolean>("cmd_apply_autostart", { enabled }),
  backupDatabase: () => call<string>("cmd_backup_database"),
  triggerTestToast: (
    questTitle: string,
    shortDescription: string,
    estimatedMins: number,
    xpReward: number
  ) =>
    call<void>("cmd_trigger_test_toast", {
      questTitle,
      shortDescription,
      estimatedMins,
      xpReward,
    }),
  createCampaign: (input: CreateCampaignInput) =>
    call<Campaign>("cmd_create_campaign", input),
  updateCampaign: (input: UpdateCampaignInput) =>
    call<Campaign>("cmd_update_campaign", input),
  archiveCampaign: (input: ArchiveCampaignInput) =>
    call<Campaign>("cmd_archive_campaign", input),
  createEpisode: (input: CreateEpisodeInput) =>
    call<Episode>("cmd_create_episode", input),
  updateEpisode: (input: UpdateEpisodeInput) =>
    call<Episode>("cmd_update_episode", input),
  setTaskStatus: (input: SetTaskStatusInput) =>
    call<QuestTask>("cmd_set_task_status", input),
  reopenTask: (input: ReopenTaskInput) =>
    call<QuestTask>("cmd_reopen_task", input),
  getAchievementNotifications: () =>
    call<AchievementNotification[]>("cmd_get_achievement_notifications"),
  acknowledgeAchievementNotification: (input: AcknowledgeNotificationsInput) =>
    call<boolean>("cmd_acknowledge_achievement_notification", input),
  setStreakPrefs: (input: SetStreakPrefsInput) =>
    call<StreakRecord>("cmd_set_streak_prefs", input),
  youtubeSetClientCredentials: (input: YoutubeClientCredentialsInput) =>
    call<void>("cmd_youtube_set_client_credentials", input),
  youtubeHasClientCredentials: () =>
    call<boolean>("cmd_youtube_has_client_credentials"),
  youtubeConnect: (loginHint?: string) =>
    call<YoutubeStatus>("cmd_youtube_connect", loginHint ? { loginHint } : {}),
  youtubeSyncNow: () => call<YoutubeStatus>("cmd_youtube_sync_now"),
  youtubeDisconnect: () => call<YoutubeStatus>("cmd_youtube_disconnect"),
  createReminder: (input: CreateReminderInput) =>
    call<ReminderConfig>("cmd_create_reminder", input),
  updateReminder: (input: UpdateReminderInput) =>
    call<ReminderConfig>("cmd_update_reminder", input),
  setReminderEnabled: (reminderId: string, enabled: boolean) =>
    call<ReminderConfig>("cmd_set_reminder_enabled", { reminderId, enabled }),
  deleteReminder: (reminderId: string) =>
    call<void>("cmd_delete_reminder", { reminderId }),
  snoozeReminder: (reminderId: string, minutes: number) =>
    call<ReminderConfig>("cmd_snooze_reminder", { reminderId, minutes }),
};
