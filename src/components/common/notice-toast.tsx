import { CheckCircle2, X, XCircle } from "lucide-react";
import { useEffect } from "react";

interface Notice {
  kind: "success" | "error";
  message: string;
}

export function NoticeToast({
  notice,
  onDismiss,
}: {
  notice: Notice | null;
  onDismiss: () => void;
}) {
  useEffect(() => {
    if (!notice) return undefined;
    const timer = window.setTimeout(onDismiss, 6000);
    return () => window.clearTimeout(timer);
  }, [notice, onDismiss]);

  if (!notice) return null;

  const isSuccess = notice.kind === "success";
  return (
    <div className="fixed bottom-5 right-5 z-50 w-[min(420px,calc(100vw-2.5rem))] animate-slide-up">
      <div
        className={`flex items-start gap-3 rounded-2xl border px-4 py-3 shadow-2xl backdrop-blur ${
          isSuccess
            ? "border-emerald-400/25 bg-emerald-950/85 text-emerald-100"
            : "border-red-400/25 bg-red-950/85 text-red-100"
        }`}
      >
        {isSuccess ? (
          <CheckCircle2 className="mt-0.5 h-5 w-5 shrink-0 text-emerald-300" />
        ) : (
          <XCircle className="mt-0.5 h-5 w-5 shrink-0 text-red-300" />
        )}
        <p className="flex-1 break-words text-sm leading-relaxed">{notice.message}</p>
        <button
          type="button"
          onClick={onDismiss}
          className="rounded-md p-1 text-white/50 transition hover:bg-white/10 hover:text-white"
          aria-label="Dismiss notification"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
