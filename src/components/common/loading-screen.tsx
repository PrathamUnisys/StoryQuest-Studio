import { Loader2 } from "lucide-react";

export function LoadingScreen({ label = "Opening your studio…" }: { label?: string }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 text-slate-400">
      <Loader2 className="h-8 w-8 animate-spin text-amber-400" />
      <p className="text-sm">{label}</p>
    </div>
  );
}
