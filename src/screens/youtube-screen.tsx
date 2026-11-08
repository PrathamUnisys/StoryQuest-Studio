import { ExternalLink, Eye, Film, ShieldCheck, Youtube } from "lucide-react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

export function YouTubeScreen() {
  const { bootstrap } = useStoryQuestStore();
  if (!bootstrap) return null;

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Distribution"
        title="YouTube Analytics"
        description="Official YouTube Data and Analytics API integration arrives in Milestone 4. Until a channel is connected, every metric is shown as unavailable — never estimated."
        actions={
          <Badge variant={bootstrap.youtube.connected ? "success" : "muted"}>
            <Youtube className="h-3 w-3" />
            {bootstrap.youtube.connected ? "Connected" : "Not connected"}
          </Badge>
        }
      />

      <Card>
        <CardHeader>
          <CardTitle>Connection status</CardTitle>
          <CardDescription>{bootstrap.youtube.note}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
              <p className="text-[10px] uppercase tracking-wider text-slate-600">Channel</p>
              <p className="mt-1 text-sm text-slate-300">
                {bootstrap.youtube.channelTitle ?? "No channel connected"}
              </p>
            </div>
            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
              <p className="text-[10px] uppercase tracking-wider text-slate-600">Last sync</p>
              <p className="mt-1 text-sm text-slate-300">
                {bootstrap.youtube.lastSyncAt ?? "Never"}
              </p>
            </div>
          </div>
          <div className="flex items-start gap-2 rounded-xl border border-sky-400/15 bg-sky-400/[0.04] p-3 text-xs leading-relaxed text-sky-200/80">
            <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-sky-300" />
            Credentials will be stored in the Windows Credential Manager. No API keys are written
            to SQLite or the project folder.
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4 sm:grid-cols-3">
        {[
          { icon: Film, label: "Published videos", value: bootstrap.creatorProgress.publishedVideosTotal },
          { icon: Eye, label: "Channel views", value: bootstrap.creatorProgress.totalChannelViews },
          { icon: Youtube, label: "Watch minutes", value: bootstrap.creatorProgress.totalWatchMinutes },
        ].map((item) => (
          <Card key={item.label}>
            <CardContent className="p-5">
              <div className="flex items-center justify-between">
                <p className="text-[10px] uppercase tracking-wider text-slate-600">{item.label}</p>
                <item.icon className="h-4 w-4 text-slate-600" />
              </div>
              <p className="mt-2 text-2xl font-semibold text-slate-200">
                {item.value ?? "Unavailable"}
              </p>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card>
        <CardContent className="p-5">
          <EmptyState
            icon={Youtube}
            title="Channel analytics are not connected"
            description="Milestone 4 adds OAuth 2.0 with a loopback redirect and PKCE, encrypted token storage, Data API statistics, Analytics API watch time, and manual publication tracking for videos published outside the studio."
            badge="Milestone 4"
          />
        </CardContent>
      </Card>

      <Card className="border-dashed">
        <CardHeader>
          <CardTitle className="text-slate-300">Known API limits</CardTitle>
          <CardDescription>
            The official Analytics API does not expose thumbnail impressions or click-through rate.
            Those values will be marked unavailable rather than inferred. Daily quota is limited;
            the sync schedule and caching are designed around that limit in Milestone 4.
          </CardDescription>
        </CardHeader>
      </Card>

      <p className="flex items-center gap-2 text-xs text-slate-600">
        <ExternalLink className="h-3.5 w-3.5" />
        Reference: YouTube Data API and YouTube Analytics API official documentation.
      </p>
    </div>
  );
}
