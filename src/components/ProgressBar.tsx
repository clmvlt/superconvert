import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";

interface ProgressBarProps {
  progress: number;
  isConverting: boolean;
}

export default function ProgressBar({ progress, isConverting }: ProgressBarProps) {
  if (!isConverting && progress === 0) return null;

  const pct = Math.round(progress * 100);
  const isDone = !isConverting && progress > 0;

  return (
    <div className="space-y-1.5">
      <div className="flex justify-between text-sm">
        <span className="text-muted-foreground">
          {isDone ? "Conversion complete" : "Converting..."}
        </span>
        <span className="text-muted-foreground tabular-nums">{pct}%</span>
      </div>
      <Progress
        value={pct}
        className={cn(
          "h-2",
          isDone && "[&>[data-slot=progress-indicator]]:bg-green-500"
        )}
      />
    </div>
  );
}
