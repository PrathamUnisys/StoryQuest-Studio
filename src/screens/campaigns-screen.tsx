import { FolderKanban, Loader2, Plus, RefreshCw, Trash2, Edit3, Save, X } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Select } from "../components/ui/select";
import { Textarea } from "../components/ui/input";
import { Separator } from "../components/ui/separator";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Progress } from "../components/ui/progress";
import { api } from "../lib/commands";
import { formatPercent } from "../lib/format";
import { useStoryQuestStore } from "../store/useStoryQuestStore";
import type { Campaign } from "../types/storyquest";

const STATUS_OPTIONS: string[] = ["planning", "active", "paused", "completed", "archived"];
const GENRE_OPTIONS: string[] = [
  "Dark Fantasy",
  "Sci-Fi",
  "Comedy",
  "Horror",
  "Adventure",
  "Drama",
  "Mystery",
];

export function CampaignsScreen() {
  const {
    setScreen,
    createCampaign,
    updateCampaign,
    archiveCampaign,
    loadBootstrap,
  } = useStoryQuestStore();
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [episodes, setEpisodes] = useState<Record<string, any[]>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [form, setForm] = useState<{
    title: string;
    synopsis: string;
    genre: string;
    visualStyle: string;
    targetRuntimeMinutes: string;
  }>({
    title: "",
    synopsis: "",
    genre: "Dark Fantasy",
    visualStyle: "2.5D Stylized Cel-Shaded",
    targetRuntimeMinutes: "8",
  });

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await api.listCampaigns();
      setCampaigns(list);
      const entries = await Promise.all(
        list.map(async (campaign) => {
          const campaignEpisodes = await api.listEpisodes(campaign.id);
          return [campaign.id, campaignEpisodes] as const;
        })
      );
      setEpisodes(Object.fromEntries(entries));
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const handleCreate = async () => {
    if (!form.title.trim()) {
      setError("Title is required.");
      return;
    }
    setCreating(true);
    setError(null);
    try {
      await createCampaign({
        title: form.title,
        synopsis: form.synopsis,
        genre: form.genre,
        visualStyle: form.visualStyle,
        targetRuntimeMinutes: Number(form.targetRuntimeMinutes) || 8,
        scaffoldFolders: true,
      });
      setEditingId(null);
      setForm({
        title: "",
        synopsis: "",
        genre: "Dark Fantasy",
        visualStyle: "2.5D Stylized Cel-Shaded",
        targetRuntimeMinutes: "8",
      });
      void load();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setCreating(false);
    }
  };

  const handleUpdate = async (campaign: Campaign) => {
    setError(null);
    try {
      await updateCampaign({
        id: campaign.id,
        title: form.title.trim() || campaign.title,
        synopsis: form.synopsis.trim() || campaign.synopsis,
        genre: form.genre,
        visualStyle: form.visualStyle,
        targetRuntimeMinutes: Number(form.targetRuntimeMinutes) || campaign.targetRuntimeMinutes,
      });
      setEditingId(null);
      void load();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const handleArchive = async (id: string) => {
    if (!confirm("Archive this campaign? Its data will be preserved but hidden from the active list.")) return;
    setError(null);
    try {
      await archiveCampaign({ id });
      void load();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const startEdit = (campaign: Campaign) => {
    setForm({
      title: campaign.title,
      synopsis: campaign.synopsis,
      genre: campaign.genre,
      visualStyle: campaign.visualStyle,
      targetRuntimeMinutes: String(campaign.targetRuntimeMinutes),
    });
    setEditingId(campaign.id);
    setError(null);
  };

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Production"
        title="Production Campaigns"
        description="Create and manage animated series campaigns. Each campaign holds episodes and a 15-stage roadmap."
        actions={
          <Button variant="outline" size="sm" onClick={() => void load()} disabled={loading}>
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            Refresh
          </Button>
        }
      />

      {error ? (
        <Card>
          <CardContent className="p-4 text-sm text-red-300">{error}</CardContent>
        </Card>
      ) : null}

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2.5">
          <CardTitle className="text-slate-200 flex items-center gap-2">
            <Plus className="h-4 w-4" /> Create Campaign
          </CardTitle>
          <Button
            variant={creating ? "destructive" : "secondary"}
            size="sm"
            onClick={() => {
              if (editingId === null) setEditingId("new");
              else setEditingId(null);
            }}
            disabled={creating}
          >
            {editingId === null ? "New Campaign" : "Cancel"}
          </Button>
        </CardHeader>
        {editingId === "new" ? (
          <CardContent className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <Label>Title</Label>
                <Input
                  value={form.title}
                  onChange={(e) => setForm({ ...form, title: e.target.value })}
                  placeholder="Chronicles of Aethelgard"
                />
              </div>
              <div className="space-y-2">
                <Label>Genre</Label>
                <Select
                  value={form.genre}
                  onChange={(e) => setForm({ ...form, genre: e.target.value })}
                >
                  {GENRE_OPTIONS.map((g) => (
                    <option key={g} value={g}>
                      {g}
                    </option>
                  ))}
                </Select>
              </div>
              <div className="space-y-2">
                <Label>Visual Style</Label>
                <Select
                  value={form.visualStyle}
                  onChange={(e) => setForm({ ...form, visualStyle: e.target.value })}
                >
                  <option value="2.5D Stylized Cel-Shaded">2.5D Stylized Cel-Shaded</option>
                  <option value="3D Realistic">3D Realistic</option>
                  <option value="Hand-Drawn 2D">Hand-Drawn 2D</option>
                  <option value="Stop Motion">Stop Motion</option>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>Target Runtime (minutes)</Label>
                <Input
                  type="number"
                  value={form.targetRuntimeMinutes}
                  onChange={(e) => setForm({ ...form, targetRuntimeMinutes: e.target.value })}
                  placeholder="8"
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label>Synopsis</Label>
              <Textarea
                value={form.synopsis}
                onChange={(e) => setForm({ ...form, synopsis: e.target.value })}
                placeholder="Describe your animated series…"
                className="min-h-[90px]"
              />
            </div>
            <div className="flex gap-2">
              <Button onClick={handleCreate} disabled={creating}>
                {creating ? "Creating…" : "Create Campaign"}
              </Button>
              <Button variant="outline" onClick={() => setEditingId(null)}>
                Cancel
              </Button>
            </div>
          </CardContent>
        ) : null}
        <CardFooter>
          {campaigns.length === 0 ? (
            <p className="text-xs text-slate-500 text-center">
              No campaigns yet. Create your first animated series above.
            </p>
          ) : (
            <p className="text-xs text-slate-500 text-center">
              {campaigns.length} campaign{campaigns.length === 1 ? "" : "s"} in your studio.
            </p>
          )}
        </CardFooter>
      </Card>

      <Separator className="my-4" />

      <Card>
        <CardHeader>
          <CardTitle className="text-slate-200">Campaigns</CardTitle>
          <CardDescription>
            Click an episode to open its Quest Map. Archive removes a campaign from the active view.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex items-center gap-2 text-sm text-slate-500">
              <Loader2 className="h-4 w-4 animate-spin" /> Loading campaigns…
            </div>
          ) : campaigns.length === 0 ? (
            <p className="text-sm text-slate-500 text-center">No campaigns found.</p>
          ) : (
            <div className="grid gap-5 xl:grid-cols-2">
              {campaigns.map((campaign) => {
                const campaignEpisodes = (episodes[campaign.id] ?? []) as any[];
                const isEditing = editingId === campaign.id;
                return (
                  <Card key={campaign.id} className={isEditing ? "ring-2 ring-amber-500/50" : ""}>
                    {isEditing ? (
                      <CardContent className="space-y-4">
                        <div className="grid gap-4 sm:grid-cols-2">
                          <div className="space-y-2">
                            <Label>Title</Label>
                            <Input
                              value={form.title}
                              onChange={(e) => setForm({ ...form, title: e.target.value })}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>Genre</Label>
                            <Select
                              value={form.genre}
                              onChange={(e) => setForm({ ...form, genre: e.target.value })}
                            >
                              {GENRE_OPTIONS.map((g) => (
                                <option key={g} value={g}>
                                  {g}
                                </option>
                              ))}
                            </Select>
                          </div>
                          <div className="space-y-2">
                            <Label>Visual Style</Label>
                            <Select
                              value={form.visualStyle}
                              onChange={(e) => setForm({ ...form, visualStyle: e.target.value })}
                            >
                              <option value="2.5D Stylized Cel-Shaded">2.5D Stylized Cel-Shaded</option>
                              <option value="3D Realistic">3D Realistic</option>
                              <option value="Hand-Drawn 2D">Hand-Drawn 2D</option>
                              <option value="Stop Motion">Stop Motion</option>
                            </Select>
                          </div>
                          <div className="space-y-2">
                            <Label>Target Runtime (minutes)</Label>
                            <Input
                              type="number"
                              value={form.targetRuntimeMinutes}
                              onChange={(e) => setForm({ ...form, targetRuntimeMinutes: e.target.value })}
                            />
                          </div>
                        </div>
                        <div className="space-y-2">
                          <Label>Synopsis</Label>
                          <Textarea
                            value={form.synopsis}
                            onChange={(e) => setForm({ ...form, synopsis: e.target.value })}
                            className="min-h-[90px]"
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Status</Label>
                          <span className="block rounded-xl border border-white/[0.06] bg-white/[0.02] px-3 py-2 text-xs text-slate-400">
                            {campaign.status}
                          </span>
                          <p className="text-[10px] text-slate-500">
                            Status and folder path can be edited from the campaign details panel.
                          </p>
                        </div>
                        <div className="flex gap-2">
                          <Button onClick={() => handleUpdate(campaign)} disabled={creating}>
                            Save
                          </Button>
                          <Button variant="outline" onClick={() => setEditingId(null)}>
                            Cancel
                          </Button>
                        </div>
                      </CardContent>
                    ) : (
                      <>
                        <CardHeader className="space-y-3">
                          <div className="flex flex-wrap items-start justify-between gap-2">
                            <div>
                              <CardTitle className="font-display text-xl">{campaign.title}</CardTitle>
                              <CardDescription className="mt-1">
                                {campaign.genre} · {campaign.visualStyle}
                              </CardDescription>
                            </div>
                            <div className="flex flex-wrap gap-1.5">
                              {campaign.isSample ? <Badge variant="sample">Sample</Badge> : null}
                              <Badge variant="secondary">{campaign.status}</Badge>
                            </div>
                          </div>
                          <p className="text-sm leading-relaxed text-slate-400">{campaign.synopsis}</p>
                        </CardHeader>
                        <CardContent className="space-y-4">
                          <div className="grid gap-3 sm:grid-cols-3">
                            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                              <p className="text-[10px] uppercase tracking-wider text-slate-600">Runtime</p>
                              <p className="mt-0.5 text-sm text-slate-300">
                                {campaign.targetRuntimeMinutes} min target
                              </p>
                            </div>
                            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                              <p className="text-[10px] uppercase tracking-wider text-slate-600">Episodes</p>
                              <p className="mt-0.5 text-sm text-slate-300">{campaignEpisodes.length}</p>
                            </div>
                            <div className="rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                              <p className="text-[10px] uppercase tracking-wider text-slate-600">Progress</p>
                              <p className="mt-0.5 text-sm text-slate-300">
                                {formatPercent(campaign.progressPercent)}
                              </p>
                            </div>
                          </div>

                          <Progress value={campaign.progressPercent} />

                          {campaignEpisodes.length ? (
                            <div className="space-y-2">
                              {campaignEpisodes.map((episode) => (
                                <div
                                  key={episode.id}
                                  className="flex items-center justify-between gap-3 rounded-xl border border-white/[0.06] bg-white/[0.02] px-3 py-2.5"
                                >
                                  <div className="min-w-0">
                                    <p className="truncate text-sm font-medium text-slate-300">
                                      {episode.title}
                                    </p>
                                    <p className="text-xs text-slate-600">
                                      Stage {episode.current_stage} of 15 · {episode.status.replace("_", " ")}
                                    </p>
                                  </div>
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => setScreen("quest-map")}
                                  >
                                    Quest Map
                                  </Button>
                                </div>
                              ))}
                            </div>
                          ) : (
                            <p className="text-xs text-slate-600">No episodes in this campaign.</p>
                          )}

                          <div className="rounded-xl border border-white/[0.05] bg-obsidian-900/40 px-3 py-2">
                            <p className="break-all text-[11px] text-slate-600">
                              {campaign.projectFolderPath}
                            </p>
                          </div>
                        </CardContent>
                        <CardFooter className="flex gap-2">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => startEdit(campaign)}
                          >
                            <Edit3 className="h-3 w-3" /> Edit
                          </Button>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleArchive(campaign.id)}
                          >
                            <Trash2 className="h-3 w-3" /> Archive
                          </Button>
                        </CardFooter>
                      </>
                    )}
                  </Card>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
