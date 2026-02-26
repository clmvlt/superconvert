import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface ProgressBarProps {
  progress: number;
  isConverting: boolean;
}

export default function ProgressBar({ progress, isConverting }: ProgressBarProps) {
  const { t } = useTranslation();

  if (!isConverting && progress === 0) return null;

  const pct = Math.round(progress * 100);
  const isDone = !isConverting && progress > 0;

  return (
    <div className="space-y-1.5">
      <div className="flex justify-between text-[11px]">
        <span className="text-muted-foreground/60">
          {isDone ? t("progress.complete") : t("progress.converting")}
        </span>
        <span className="text-muted-foreground/60 tabular-nums font-medium">{pct}%</span>
      </div>
      <div className="h-1.5 bg-muted/30 rounded-full overflow-hidden">
        <div
          className={cn(
            "h-full rounded-full transition-all duration-500 ease-out",
            isDone ? "bg-green-500/70" : "bg-primary/60"
          )}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
