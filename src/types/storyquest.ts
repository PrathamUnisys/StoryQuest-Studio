export type ScreenId =
  | "dashboard"
  | "journey"
  | "campaigns"
  | "quest-map"
  | "daily-quests"
  | "youtube"
  | "achievements"
  | "assets"
  | "calendar"
  | "ai-planner"
  | "settings";

export interface UserProfile {
  id: string;
  displayName: string;
  studioName: string;
  currentLevel: number;
  totalXp: number;
  activeTitle: string;
  activeTheme: string;
  avatarBadge: string;
  dailyXpCustomTasks: number;
  lastCustomXpDate: string | null;
}

export interface CreatorRank {
  level: number;
  rankName: string;
  minPublishedVideos: number;
  badgeIcon: string;
  description: string;
}

export interface CreatorProgress {
  userId: string;
  rankLevel: number;
  publishedVideosTotal: number;
  shortsCount: number;
  longFormCount: number;
  subscriberCount: number | null;
  totalChannelViews: number | null;
  totalWatchMinutes: number | null;
  lastRecalculatedAt: string | null;
  updatedAt: string;
}

export interface Campaign {
  id: string;
  title: string;
  synopsis: string;
  genre: string;
  visualStyle: string;
  worldInfo: string;
  targetRuntimeMinutes: number;
  status: string;
  projectFolderPath: string;
  progressPercent: number;
  isSample: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface Episode {
  id: string;
  campaignId: string;
  episodeNumber: number;
  title: string;
  premise: string;
  narrativeStructure: string;
  targetDurationSec: number;
  currentStage: number;
  roadmapApproved: boolean;
  status: string;
  youtubeUrl: string | null;
}

export interface QuestTask {
  id: string;
  campaignId: string;
  episodeId: string;
  stageNumber: number;
  stageName: string;
  title: string;
  description: string;
  objective: string;
  requiredInput: string;
  aiInstructions: string;
  humanResponsibilities: string;
  requiredSoftware: string;
  expectedOutput: string;
  estimatedEffortMins: number;
  actualEffortMins: number;
  definitionOfDone: string;
  completionRequirementType: string;
  completionArtifactValue: string;
  difficulty: string;
  xpReward: number;
  skillCategory: string;
  status: string;
  isDailyQuest: boolean;
  dueDate: string | null;
  notes: string;
  attachedFilesJson: string;
  dependencies: string[];
  completedAt: string | null;
  approvedAt: string | null;
}

export interface Achievement {
  id: string;
  code: string;
  title: string;
  description: string;
  category: string;
  xpBonus: number;
  badgeSvgKey: string;
  cosmeticReward: string | null;
  unlocked: boolean;
  unlockedAt: string | null;
  progressCurrent: number;
  progressTarget: number;
  conditionType: string;
  conditionValue: number;
  conditionScope: string;
  progressAvailable: boolean;
  isHidden: boolean;
}

export interface AchievementUnlock {
  achievementId: string;
  code: string;
  title: string;
  xpBonus: number;
}

export interface AchievementNotification {
  id: string;
  achievementId: string | null;
  kind: string;
  title: string;
  body: string;
  itemsJson: string;
  presentedAt: string | null;
  acknowledgedAt: string | null;
}

export interface SetStreakPrefsInput {
  weeklyRestDay: number;
  vacationMode: boolean;
  vacationUntil: string | null;
}

export interface AcknowledgeNotificationsInput {
  notificationIds: string[];
}

export interface ActivityDay {
  date: string;
  xp: number;
  tasksCompleted: number;
  focusMinutes: number;
  active: boolean;
  status: string;
}

export interface SettingItem {
  key: string;
  value: string;
  category: string;
}

export interface AppInfo {
  name: string;
  version: string;
  identifier: string;
  dataDir: string;
  dbPath: string;
  schemaVersion: string;
}

export interface TaskCounts {
  total: number;
  locked: number;
  available: number;
  inProgress: number;
  review: number;
  approved: number;
  completed: number;
  blocked: number;
}

export interface StreakRecord {
  id: string;
  userId: string;
  currentStreak: number;
  bestStreak: number;
  streakFreezesAvailable: number;
  weeklyRestDay: number;
  vacationMode: boolean;
  vacationUntil: string | null;
  lastProductiveDate: string | null;
  historyJson: string;
}

export interface ActivityDay {
  date: string;
  xp: number;
  tasksCompleted: number;
  focusMinutes: number;
  active: boolean;
}

export interface YoutubeStatus {
  connected: boolean;
  channelTitle: string | null;
  channelId: string | null;
  lastSyncAt: string | null;
  note: string;
  hasClientCredentials: boolean;
}

export interface YoutubeClientCredentialsInput {
  clientId: string;
  clientSecret: string;
}

export interface CreateReminderInput {
  title: string;
  reminderType: string;
  taskId: string | null;
  scheduleTime: string;
  daysOfWeek: number[];
  specificDate: string | null;
}

export interface UpdateReminderInput extends CreateReminderInput {
  id: string;
  enabled: boolean;
}

export interface StoryPlannerInput {
  campaignId: string;
  episodeId: string;
  storyTitle: string;
  shortPremise: string;
  genre: string | null;
  desiredRuntimeMins: number;
  creativeInstructions: string | null;
  targetStageLimit: number | null;
}

export interface GeneratedRoadmap {
  storyAnalysis: string;
  improvedPremise: string;
  narrativeStructure: string;
  generatedTasks: QuestTask[];
  source: string;
}

export interface AiProviderConfig {
  id: string;
  providerName: string;
  displayName: string;
  baseUrl: string;
  defaultModel: string;
  credentialKeyName: string;
  isActive: boolean;
  temperature: number;
  maxTokens: number;
  hasApiKey: boolean;
}

export interface UpdateAiProviderInput {
  providerName: string;
  baseUrl: string;
  defaultModel: string;
  temperature: number;
  maxTokens: number;
}

export interface ReminderConfig {
  id: string;
  title: string;
  reminderType: string;
  taskId: string | null;
  scheduleTime: string;
  daysOfWeekJson: string;
  specificDate: string | null;
  enabled: boolean;
  snoozedUntil: string | null;
  windowsTaskName: string;
  lastTriggeredAt: string | null;
}

export interface BootstrapState {
  profile: UserProfile;
  creatorProgress: CreatorProgress;
  creatorRank: CreatorRank;
  creatorRanks: CreatorRank[];
  settings: SettingItem[];
  activeCampaign: Campaign | null;
  activeEpisode: Episode | null;
  taskCounts: TaskCounts;
  episodeProgressPercent: number;
  streak: StreakRecord;
  achievements: Achievement[];
  deadlines: QuestTask[];
  weeklyActivity: ActivityDay[];
  youtube: YoutubeStatus;
  appInfo: AppInfo;
  today: string;
  onboardingCompleted: boolean;
}

export interface ThresholdUpdate {
  level: number;
  minPublishedVideos: number;
}

export interface CreateCampaignInput {
  title: string;
  synopsis: string;
  genre?: string;
  visualStyle?: string;
  targetRuntimeMinutes?: number;
  scaffoldFolders?: boolean;
}

export interface UpdateCampaignInput {
  id: string;
  title?: string;
  synopsis?: string;
  genre?: string;
  visualStyle?: string;
  targetRuntimeMinutes?: number;
  status?: string;
  projectFolderPath?: string;
}

export interface ArchiveCampaignInput {
  id: string;
}

export interface CreateEpisodeInput {
  campaignId: string;
  title: string;
  premise?: string;
  episodeNumber?: number;
  targetDurationSec?: number;
}

export interface UpdateEpisodeInput {
  id: string;
  title?: string;
  premise?: string;
  narrativeStructure?: string;
  targetDurationSec?: number;
  currentStage?: number;
  roadmapApproved?: boolean;
  status?: string;
  youtubeUrl?: string;
}

export interface SetTaskStatusInput {
  taskId: string;
  status: string;
  notes?: string;
  attachedFilesJson?: string;
  actualEffortMins?: number;
  completionArtifactValue?: string;
}

export interface ReopenTaskInput {
  taskId: string;
}
