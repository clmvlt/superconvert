import { FolderOpen, Loader2, RotateCcw, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ConvertButtonProps {
  fileCount: number;
  isConverting: boolean;
  conversionDone: boolean;
  onConvert: () => void;
  onClear: () => void;
  onOpenFolder: () => void;
  onReset: () => void;
}

export default function ConvertButton({
  fileCount,
  isConverting,
  conversionDone,
  onConvert,
  onClear,
  onOpenFolder,
  onReset,
}: ConvertButtonProps) {
  if (conversionDone) {
    return (
      <div className="flex flex-col gap-2">
        <Button
          onClick={onOpenFolder}
          className="w-full bg-green-600 hover:bg-green-500 text-white"
          size="lg"
        >
          <FolderOpen className="size-4" />
          Open output folder
        </Button>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            onClick={onReset}
            className="flex-1"
          >
            <RotateCcw className="size-4" />
            Convert again
          </Button>
          <Button
            variant="ghost"
            onClick={onClear}
          >
            <Trash2 className="size-4" />
            Clear
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex gap-2">
      <Button
        onClick={onConvert}
        disabled={fileCount === 0 || isConverting}
        className="flex-1"
        size="lg"
      >
        {isConverting ? (
          <>
            <Loader2 className="size-4 animate-spin" />
            Converting...
          </>
        ) : (
          `Convert ${fileCount} file${fileCount !== 1 ? "s" : ""}`
        )}
      </Button>
      {fileCount > 0 && !isConverting && (
        <Button
          variant="destructive"
          onClick={onClear}
          size="lg"
        >
          <Trash2 className="size-4" />
        </Button>
      )}
    </div>
  );
}
