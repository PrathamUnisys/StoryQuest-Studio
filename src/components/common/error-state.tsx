import { AlertTriangle, RotateCcw } from "lucide-react";

import { Button } from "../ui/button";

export function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 px-6 text-center">
      <div className="rounded-2xl border border-red-400/20 bg-red-400/[0.06] p-3 text-red-300">
        <AlertTriangle className="h-7 w-7" />
      </div>
      <div>
        <h2 className="font-display text-xl font-semibold text-slate-100">
          The studio could not open
        </h2>
        <p className="mx-auto mt-2 max-w-lg text-sm leading-relaxed text-slate-400">{message}</p>
      </div>
      <Button variant="secondary" onClick={onRetry}>
        <RotateCcw className="h-4 w-4" />
        Try again
      </Button>
    </div>
  );
}
