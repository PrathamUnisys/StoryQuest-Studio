import { useState } from "react";
import { ExternalLink, Eye, Film, KeyRound, RefreshCw, ShieldCheck, Unplug, Youtube } from "lucide-react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

export function YouTubeScreen() {
  const { bootstrap, saving, youtubeSetClientCredentials, youtubeConnect, youtubeSyncNow, youtubeDisconnect } =
    useStoryQuestStore();
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [loginHint, setLoginHint] = useState("");
  const [busy, setBusy] = useState(false);
  if (!bootstrap) return null;

  const youtube = bootstrap.youtube;
  const progress = bootstrap.creatorProgress;

  async function run(fn: () => Promise<void>) {
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Distribution"
        title="YouTube Analytics"
        description="Official YouTube Data and Analytics APIs via Google OAuth 2.0 (PKCE + loopback). Metrics are real API values — never estimated."
        actions={
          <Badge variant={youtube.connected ? "success" : "muted"}>
            <Youtube className="h-3 w-3" />
            {youtube.connected ? "Connected" : "Not connected"}
          </Badge>
        }
      />

      <Card>
        <CardHeader>
          <CardTitle>Connection</CardTitle>
          <CardDescription>{youtube.note}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-3 sm:grid-cols-3">
            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
              <p className="text-[10px] uppercase tracking-wider text-slate-600">Channel</p>
              <p className="mt-1 text-sm text-slate-300">{youtube.channelTitle ?? "No channel connected"}</p>
            </div>
            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
              <p className="text-[10px] uppercase tracking-wider text-slate-600">Channel ID</p>
              <p className="mt-1 text-sm text-slate-300">{youtube.channelId ?? "—"}</p>
            </div>
            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
              <p className="text-[10px] uppercase tracking-wider text-slate-600">Last sync</p>
              <p className="mt-1 text-sm text-slate-300">{youtube.lastSyncAt ?? "Never"}</p>
            </div>
          </div>

          {!youtube.hasClientCredentials && (
            <div className="space-y-3 rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-4">
              <p className="flex items-center gap-2 text-sm font-medium text-slate-200">
                <KeyRound className="h-4 w-4 text-sky-300" /> Google OAuth credentials
              </p>
              <p className="text-xs leading-relaxed text-slate-400">
                Create a <strong>Desktop app</strong> OAuth client in Google Cloud Console, enable YouTube Data API v3 +
                YouTube Analytics API, then paste the Client ID / Secret here — or set{" "}
                <code className="text-slate-200">GOOGLE_YOUTUBE_CLIENT_ID</code> /{" "}
                <code className="text-slate-200">GOOGLE_YOUTUBE_CLIENT_SECRET</code> in the project <code className="text-slate-200">.env</code>{" "}
                (see <code className="text-slate-200">.env.example</code>) and restart. Saved credentials go to Windows
                Credential Manager, never SQLite.
              </p>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="space-y-1.5">
                  <Label htmlFor="yt-client-id">Client ID</Label>
                  <Input
                    id="yt-client-id"
                    value={clientId}
                    onChange={(e) => setClientId(e.target.value)}
                    placeholder="…apps.googleusercontent.com"
                    autoComplete="off"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="yt-client-secret">Client Secret</Label>
                  <Input
                    id="yt-client-secret"
                    type="password"
                    value={clientSecret}
                    onChange={(e) => setClientSecret(e.target.value)}
                    autoComplete="off"
                  />
                </div>
              </div>
              <Button
                disabled={busy || saving || !clientId.trim() || !clientSecret.trim()}
                onClick={() => run(() => youtubeSetClientCredentials(clientId.trim(), clientSecret.trim()))}
              >
                Save credentials
              </Button>
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="yt-login-hint">Connect as (optional — forces the right Google account)</Label>
            <div className="flex flex-wrap items-end gap-2">
              <div className="w-64">
                <Input
                  id="yt-login-hint"
                  type="email"
                  value={loginHint}
                  onChange={(e) => setLoginHint(e.target.value)}
                  placeholder="creator@gmail.com"
                  autoComplete="off"
                />
              </div>
              <Button disabled={busy || saving || !youtube.hasClientCredentials || youtube.connected}
                onClick={() => run(() => youtubeConnect(loginHint.trim() || undefined))}>
                <Youtube className="h-4 w-4" /> Connect YouTube
              </Button>
            </div>
            <p className="text-xs text-slate-500">
              The browser will show an account chooser (prompt select_account). If your channel account isn't listed,
              click <span className="text-slate-300">Use another account</span> / <span className="text-slate-300">Add account</span> in
              Google's page — or paste the auth link into an incognito window. This account must be a Test user in Google
              Cloud Console while the app is in Testing.
            </p>
          </div>

          <div className="flex flex-wrap gap-2">
            <Button variant="secondary" disabled={busy || saving || !youtube.connected}
              onClick={() => run(youtubeSyncNow)}>
              <RefreshCw className="h-4 w-4" /> Sync now
            </Button>
            <Button variant="outline" disabled={busy || saving || !youtube.connected}
              onClick={() => run(youtubeDisconnect)}>
              <Unplug className="h-4 w-4" /> Disconnect
            </Button>
          </div>

          <div className="flex items-start gap-2 rounded-xl border border-sky-400/15 bg-sky-400/[0.04] p-3 text-xs leading-relaxed text-sky-200/80">
            <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-sky-300" />
            OAuth tokens are stored only in Windows Credential Manager. Requested scopes: youtube.readonly,
            yt-analytics.readonly, yt-analytics-monetary.readonly. Thumbnail impressions / CTR are not exposed by the
            Analytics API and stay unavailable rather than inferred.
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4 sm:grid-cols-4">
        {[
          { icon: Film, label: "Published videos", value: progress.publishedVideosTotal },
          { icon: Eye, label: "Subscribers", value: progress.subscriberCount },
          { icon: Eye, label: "Channel views", value: progress.totalChannelViews },
          { icon: Youtube, label: "Watch minutes (28d)", value: progress.totalWatchMinutes },
        ].map((item) => (
          <Card key={item.label}>
            <CardContent className="p-5">
              <div className="flex items-center justify-between">
                <p className="text-[10px] uppercase tracking-wider text-slate-600">{item.label}</p>
                <item.icon className="h-4 w-4 text-slate-600" />
              </div>
              <p className="mt-2 text-2xl font-semibold text-slate-200">{item.value ?? "Unavailable"}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      {!youtube.connected && (
        <Card>
          <CardContent className="p-5">
            <EmptyState
              icon={Youtube}
              title="Channel analytics are not connected"
              description="Connect above to sync real channel statistics (Data API) and 28-day watch time / views / subscriber changes (Analytics API). Sync updates creator rank and can unlock YouTube achievements."
              badge="Milestone 4"
            />
          </CardContent>
        </Card>
      )}

      <p className="flex items-center gap-2 text-xs text-slate-600">
        <ExternalLink className="h-3.5 w-3.5" />
        Reference: YouTube Data API and YouTube Analytics API official documentation (developers.google.com/youtube).
      </p>
    </div>
  );
}
