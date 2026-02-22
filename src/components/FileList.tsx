import { Check, X, Loader2, Circle, ExternalLink, Trash2, Image, Music, FileText } from "lucide-react";
import type { ConversionFile, FileCategory } from "@/types/conversion";
import { CATEGORY_COLORS, CATEGORY_LABELS } from "@/types/conversion";
import { formatFileSize } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

interface FileListProps {
  files: ConversionFile[];
  outputFormat: string;
  onRemove: (id: string) => void;
  onOpenFile: (path: string) => void;
  disabled: boolean;
}

function CategoryIcon({ category }: { category: FileCategory }) {
  const className = "size-3.5";
  switch (category) {
    case "image":
      return <Image className={className} />;
    case "audio":
      return <Music className={className} />;
    case "document":
      return <FileText className={className} />;
  }
}

function StatusIcon({ status }: { status: ConversionFile["status"] }) {
  switch (status) {
    case "done":
      return <Check className="size-4 text-green-400" />;
    case "error":
      return <X className="size-4 text-red-400" />;
    case "converting":
      return <Loader2 className="size-4 text-blue-400 animate-spin" />;
    default:
      return <Circle className="size-3.5 text-muted-foreground/40" />;
  }
}

export default function FileList({ files, outputFormat, onRemove, onOpenFile, disabled }: FileListProps) {
  if (files.length === 0) return null;

  return (
    <div className="flex-1 min-h-0 flex flex-col">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium text-muted-foreground">
          {files.length} file{files.length > 1 ? "s" : ""}
        </h3>
      </div>
      <ScrollArea className="flex-1">
        <div className="space-y-1 pr-3">
          {files.map((file) => (
            <div
              key={file.id}
              className="flex items-center gap-3 px-3 py-2 bg-card/50 rounded-lg group hover:bg-card transition-colors"
            >
              <div className="w-5 flex-shrink-0 flex justify-center">
                <StatusIcon status={file.status} />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm text-foreground truncate">
                    {file.name}
                  </span>
                  <div className="flex items-center gap-1 flex-shrink-0">
                    <span className={`inline-flex items-center gap-1 text-[10px] px-1.5 py-0 h-4 rounded-full ${CATEGORY_COLORS[file.category]}`}>
                      <CategoryIcon category={file.category} />
                      {CATEGORY_LABELS[file.category]}
                    </span>
                    <Badge variant="secondary" className="text-[10px] px-1.5 py-0 h-4 uppercase">
                      {file.extension}
                    </Badge>
                    <span className="text-muted-foreground/40 text-xs">&rarr;</span>
                    <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-4 uppercase">
                      {outputFormat}
                    </Badge>
                  </div>
                </div>
                {file.status === "converting" && (
                  <div className="mt-1.5 h-1 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-500 rounded-full transition-all duration-300"
                      style={{ width: `${Math.round(file.progress * 100)}%` }}
                    />
                  </div>
                )}
                {file.status === "error" && file.error && (
                  <p className="text-xs text-destructive mt-0.5 truncate">
                    {file.error}
                  </p>
                )}
              </div>
              <span className="text-xs text-muted-foreground flex-shrink-0">
                {formatFileSize(file.size)}
              </span>
              {file.status === "done" && file.outputPath && (
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => onOpenFile(file.outputPath!)}
                  title="Open converted file"
                >
                  <ExternalLink className="size-3.5 text-primary" />
                </Button>
              )}
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={() => onRemove(file.id)}
                disabled={disabled}
                className="opacity-0 group-hover:opacity-100 transition-opacity"
                title="Remove"
              >
                <Trash2 className="size-3.5" />
              </Button>
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
