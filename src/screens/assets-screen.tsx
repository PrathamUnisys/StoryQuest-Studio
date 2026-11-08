import { Boxes, Images, Palette, Sparkles } from "lucide-react";

import { EmptyState } from "../components/common/empty-state";
import { PageHeader } from "../components/common/page-header";
import { Badge } from "../components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { useStoryQuestStore } from "../store/useStoryQuestStore";

export function AssetsScreen() {
  const { bootstrap } = useStoryQuestStore();
  if (!bootstrap) return null;

  return (
    <div className="animate-fade-in space-y-6">
      <PageHeader
        eyebrow="Workshop"
        title="Asset Workshop"
        description="A local library for characters, rigs, animation clips, backgrounds, and reusable production assets. This module begins in Milestone 2."
        actions={<Badge variant="muted">Milestone 2</Badge>}
      />

      <div className="grid gap-4 sm:grid-cols-3">
        {[
          { icon: Boxes, label: "Characters", value: "0" },
          { icon: Palette, label: "Environments", value: "0" },
          { icon: Images, label: "Animation clips", value: "0" },
        ].map((item) => (
          <Card key={item.label}>
            <CardContent className="flex items-center justify-between p-5">
              <div>
                <p className="text-[10px] uppercase tracking-wider text-slate-600">{item.label}</p>
                <p className="mt-1 text-2xl font-semibold text-slate-200">{item.value}</p>
              </div>
              <item.icon className="h-5 w-5 text-slate-600" />
            </CardContent>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Library is empty</CardTitle>
          <CardDescription>
            Asset records, thumbnails, rig metadata, and clip previews will be stored locally and
            linked to characters and episodes.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <EmptyState
            icon={Sparkles}
            title="No workshop assets yet"
            description="Milestone 2 adds asset creation, character and environment libraries, and links to production tasks. No stock thumbnails or fake counts are included."
            badge="Milestone 2"
          />
        </CardContent>
      </Card>
    </div>
  );
}
