import type { LucideIcon } from "lucide-react";
import {
  Award,
  CalendarClock,
  Compass,
  Film,
  FolderKanban,
  LayoutDashboard,
  ListChecks,
  Settings,
  Sparkles,
  TrendingUp,
} from "lucide-react";

import type { ScreenId } from "../types/storyquest";

export interface NavItem {
  id: ScreenId;
  label: string;
  shortLabel: string;
  icon: LucideIcon;
}

export const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", shortLabel: "Dashboard", icon: LayoutDashboard },
  { id: "journey", label: "My Journey", shortLabel: "Journey", icon: TrendingUp },
  { id: "campaigns", label: "Production Campaigns", shortLabel: "Campaigns", icon: FolderKanban },
  { id: "quest-map", label: "Quest Map", shortLabel: "Quest Map", icon: Compass },
  { id: "daily-quests", label: "Daily Quests", shortLabel: "Daily Quests", icon: ListChecks },
  { id: "youtube", label: "YouTube Analytics", shortLabel: "YouTube", icon: Film },
  { id: "achievements", label: "Creator Achievements", shortLabel: "Achievements", icon: Award },
  { id: "assets", label: "Asset Workshop", shortLabel: "Assets", icon: Sparkles },
  { id: "calendar", label: "Calendar", shortLabel: "Calendar", icon: CalendarClock },
  { id: "settings", label: "Settings", shortLabel: "Settings", icon: Settings },
];

export function screenTitle(screen: ScreenId): string {
  return NAV_ITEMS.find((item) => item.id === screen)?.label ?? "Dashboard";
}
