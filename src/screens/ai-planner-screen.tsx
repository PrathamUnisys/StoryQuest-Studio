import { useEffect, useState } from "react";
import { Bot, CheckCircle2, KeyRound, Loader2, Sparkles, Wand2 } from "lucide-react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { api } from "../lib/commands";
import { useStoryQuestStore } from "../store/useStoryQuestStore";
import type { AiProviderConfig, Campaign, Episode, GeneratedRoadmap } from "../types/storyquest";

export function AiPlannerScreen() {
  const { bootstrap } = useStoryQuestStore();
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [episodes, setEpisodes] = useState<Episode[]>([]);
  const [providers, setProviders] = useState<AiProviderConfig[]>([]);
  const [campaignId, setCampaignId] = useState("");
  const [episodeId, setEpisodeId] = useState("");
  const [storyTitle, setStoryTitle] = useState("");
  const [premise, setPremise] = useState("");
  const [genre, setGenre] = useState("");
  const [runtime, setRuntime] = useState(8);
  const [instructions, setInstructions] = useState("");
  const [roadmap, setRoadmap] = useState<GeneratedRoadmap | null>(null);
  const [busy, setBusy] = useState(false);
  const [applying, setApplying] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState<Record<string, string>>({});

  const loadProviders = async () => setProviders(await api.listAiProviders());

  useEffect(() => {
    void (async () => {
      try {
        const [camps, provs] = await Promise.all([api.listCampaigns(), loadProviders()]);
        setCampaigns(camps);
        if (camps.length > 0) setCampaignId(camps[0].id);
      } catch (e) {
        setMessage(e instanceof Error ? e.message : String(e));
      }
    })();
  }, []);

  useEffect(() => {
    if (!campaignId) return;
    void api.listEpisodes(campaignId).then((eps) => {
      setEpisodes(eps);
      if (eps.length > 0) setEpisodeId(eps[0].id);
    }).catch(() => setEpisodes([]));
  }, [campaignId]);

  const activeProvider = providers.find((p) => p.isActive);

  const generate = async () => {
    if (!campaignId || !episodeId || !storyTitle.trim() || !premise.trim()) {
      setMessage("Pick a campaign + episode, add a title and premise first.");
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      const result = await api.generateAiRoadmap({
        campaignId,
        episodeId,
        storyTitle: storyTitle.trim(),
        shortPremise: premise.trim(),
        genre: genre.trim() || null,
        desiredRuntimeMins: runtime,
        creativeInstructions: instructions.trim() || null,
        targetStageLimit: null,
      });
      setRoadmap(result);
      setMessage(
        result.source === "ai"
          ? `Roadmap generated with ${activeProvider?.displayName ?? "AI provider"}.`
          : "No AI provider available — deterministic offline template used."
      );
    } catch (e) {
      setMessage(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const apply = async () => {
    if (!roadmap) return;
    setApplying(true);
    try {
      const count = await api.applyGeneratedRoadmap(roadmap);
      setMessage(`Applied ${count} tasks to episode ${episodeId}. Check the Quest Map.`);
      setRoadmap(null);
    } catch (e) {
      setMessage(e instanceof Error ? e.message : String(e));
    } finally {
      setApplying(false);
    }
  };

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Production AI"
        title="AI Planner"
        description="Provider-agnostic roadmap generation. Uses the active AI provider when configured; otherwise falls back to the offline studio template — always labeled."
        actions={
          <Badge variant={activeProvider?.hasApiKey ? "success" : "muted"}>
            <Bot className="h-3 w-3" />
            {activeProvider ? `${activeProvider.displayName} · ${activeProvider.defaultModel}` : "Offline template"}
          </Badge>
        }
      />

      <div className="grid gap-6 xl:grid-cols-[1.2fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>New episode roadmap</CardTitle>
            <CardDescription>Describe the episode; the planner returns a 15-stage task list.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-3 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label>Campaign</Label>
                <select className="w-full rounded-md border border-white/10 bg-obsidian-900 px-2 py-2 text-sm text-slate-200"
                  value={campaignId} onChange={(e) => setCampaignId(e.target.value)}>
                  {campaigns.map((c) => <option key={c.id} value={c.id}>{c.title}</option>)}
                </select>
              </div>
              <div className="space-y-1.5">
                <Label>Episode</Label>
                <select className="w-full rounded-md border border-white/10 bg-obsidian-900 px-2 py-2 text-sm text-slate-200"
                  value={episodeId} onChange={(e) => setEpisodeId(e.target.value)}>
                  {episodes.map((ep) => <option key={ep.id} value={ep.id}>EP{ep.episodeNumber} · {ep.title}</option>)}
                </select>
              </div>
            </div>
            <div className="space-y-1.5">
              <Label>Story title</Label>
              <Input value={storyTitle} onChange={(e) => setStoryTitle(e.target.value)} placeholder="The Obsidian Relic Awakens" />
            </div>
            <div className="space-y-1.5">
              <Label>Premise</Label>
              <Input value={premise} onChange={(e) => setPremise(e.target.value)} placeholder="A cursed relic reawakens in a deserted station…" />
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label>Genre</Label>
                <Input value={genre} onChange={(e) => setGenre(e.target.value)} placeholder="Dark Fantasy Animation" />
              </div>
              <div className="space-y-1.5">
                <Label>Runtime (min)</Label>
                <Input type="number" min={1} value={runtime} onChange={(e) => setRuntime(Number(e.target.value) || 8)} />
              </div>
            </div>
            <div className="space-y-1.5">
              <Label>Creative instructions (optional)</Label>
              <Input value={instructions} onChange={(e) => setInstructions(e.target.value)} placeholder="Keep it solo-friendly, 2.5D rigs…" />
            </div>
            {message ? (
              <div className="rounded-xl border border-sky-400/20 bg-sky-400/[0.05] px-4 py-3 text-sm text-sky-200">{message}</div>
            ) : null}
            <Button disabled={busy} onClick={() => void generate()}>
              {busy ? <Loader2 className="h-4 w-4 animate-spin" /> : <Wand2 className="h-4 w-4" />}
              Generate roadmap
            </Button>
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Roadmap preview</CardTitle>
              <CardDescription>{roadmap ? `Source: ${roadmap.source === "ai" ? "AI provider" : "offline template"}` : "Generate to preview."}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {roadmap ? (
                <>
                  <div className="space-y-2">
                    <p className="text-xs leading-relaxed text-slate-400"><strong className="text-slate-200">Analysis:</strong> {roadmap.storyAnalysis}</p>
                    <p className="text-xs leading-relaxed text-slate-400"><strong className="text-slate-200">Structure:</strong> {roadmap.narrativeStructure}</p>
                  </div>
                  <div className="max-h-64 space-y-1.5 overflow-y-auto pr-1">
                    {roadmap.generatedTasks.map((t) => (
                      <div key={t.id} className="rounded-lg border border-white/[0.06] bg-obsidian-900/60 px-3 py-2 text-xs">
                        <p className="font-medium text-slate-200">{t.title}</p>
                        <p className="mt-0.5 text-slate-500">{t.stageName} · {t.estimatedEffortMins} min · +{t.xpReward} XP</p>
                      </div>
                    ))}
                  </div>
                  <Button className="w-full" disabled={applying} onClick={() => void apply()}>
                    {applying ? <Loader2 className="h-4 w-4 animate-spin" /> : <CheckCircle2 className="h-4 w-4" />}
                    Apply {roadmap.generatedTasks.length} tasks to episode
                  </Button>
                </>
              ) : (
                <EmptyState icon={Sparkles} title="No roadmap yet" description="Fill in the premise and generate. Applying writes tasks into the selected episode (episodes with existing tasks are protected)." />
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>AI providers</CardTitle>
              <CardDescription>Keys live in Windows Credential Manager — never in SQLite.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {providers.map((p) => (
                <div key={p.providerName} className="space-y-2 rounded-xl border border-white/[0.06] bg-obsidian-900/60 p-3">
                  <div className="flex items-center justify-between">
                    <p className="text-sm text-slate-200">{p.displayName}</p>
                    <Badge variant={p.isActive ? "success" : "muted"}>{p.isActive ? "Active" : p.hasApiKey ? "Key saved" : "No key"}</Badge>
                  </div>
                  <p className="text-xs text-slate-500">{p.baseUrl} · {p.defaultModel} · temp {p.temperature}</p>
                  <div className="flex gap-2">
                    <Input type="password" placeholder="API key" value={apiKeyDraft[p.providerName] ?? ""}
                      onChange={(e) => setApiKeyDraft({ ...apiKeyDraft, [p.providerName]: e.target.value })} />
                    <Button variant="secondary" size="sm"
                      disabled={!(apiKeyDraft[p.providerName] ?? "").trim()}
                      onClick={() => void api.saveApiKey(p.credentialKeyName, apiKeyDraft[p.providerName].trim()).then(() => {
                        setApiKeyDraft({ ...apiKeyDraft, [p.providerName]: "" });
                        void loadProviders();
                      }).catch((e) => setMessage(e instanceof Error ? e.message : String(e)))}>
                      <KeyRound className="h-3.5 w-3.5" /> Save
                    </Button>
                  </div>
                  <div className="flex gap-2">
                    {!p.isActive && (
                      <Button variant="outline" size="sm" onClick={() => void api.setActiveAiProvider(p.providerName).then(() => void loadProviders())}>
                        Make active
                      </Button>
                    )}
                    <Button variant="ghost" size="sm" onClick={() => void api.testAiProvider(p.providerName).then((m) => setMessage(m)).catch((e) => setMessage(e instanceof Error ? e.message : String(e)))}>
                      Test connection
                    </Button>
                  </div>
                </div>
              ))}
              {bootstrap ? null : null}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
