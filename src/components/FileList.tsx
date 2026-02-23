import { Check, X, Loader2, Circle, ExternalLink, Trash2, ArrowRightLeft } from "lucide-react";
import type { ConversionFile, FileCategory } from "@/types/conversion";
import { getOutputFormats, FORMAT_OPTIONS } from "@/types/conversion";
import { formatFileSize } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface FileListProps {
  files: ConversionFile[];
  outputFormats: Partial<Record<FileCategory, string>>;
  onRemove: (id: string) => void;
  onOpenFile: (path: string) => void;
  onConvertSingle: (fileId: string, format: string) => void;
  disabled: boolean;
}

function StatusIcon({ status }: { status: ConversionFile["status"] }) {
  switch (status) {
    case "done":
      return <Check className="size-5 text-green-400" />;
    case "error":
      return <X className="size-5 text-red-400" />;
    case "converting":
      return <Loader2 className="size-5 text-blue-400 animate-spin" />;
    default:
      return <Circle className="size-4 text-muted-foreground/30" />;
  }
}

export default function FileList({ files, outputFormats, onRemove, onOpenFile, onConvertSingle, disabled }: FileListProps) {
  if (files.length === 0) return null;

  return (
    <div className="flex-1 min-h-0 flex flex-col">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium text-muted-foreground">
          {files.length} file{files.length > 1 ? "s" : ""}
        </h3>
      </div>
      <ScrollArea className="flex-1">
        <div className="space-y-1.5 pr-3">
          {files.map((file) => {
            const targetFormat = outputFormats[file.category] ?? "?";
            const formats = getOutputFormats(file.category);
            const isBusy = file.status === "converting";
            return (
              <div
                key={file.id}
                className="flex items-center gap-3 px-4 py-3 bg-card/50 rounded-xl group hover:bg-card transition-colors"
              >
                <div className="w-6 flex-shrink-0 flex justify-center">
                  <StatusIcon status={file.status} />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2.5">
                    <span className="text-[15px] text-foreground truncate">
                      {file.name}
                    </span>
                    <div className="flex items-center gap-1.5 flex-shrink-0">
                      <Badge variant="secondary" className="text-[11px] px-2 py-0.5 uppercase font-medium">
                        {file.extension}
                      </Badge>
                      <span className="text-muted-foreground/40">&rarr;</span>
                      <Badge variant="outline" className="text-[11px] px-2 py-0.5 uppercase font-medium">
                        {targetFormat}
                      </Badge>
                    </div>
                  </div>
                  {file.status === "converting" && (
                    <div className="mt-2 h-1.5 bg-muted rounded-full overflow-hidden">
                      <div
                        className="h-full bg-blue-500 rounded-full transition-all duration-300"
                        style={{ width: `${Math.round(file.progress * 100)}%` }}
                      />
                    </div>
                  )}
                  {file.status === "error" && file.error && (
                    <p className="text-xs text-destructive mt-1 truncate">
                      {file.error}
                    </p>
                  )}
                </div>
                <span className="text-xs text-muted-foreground flex-shrink-0">
                  {formatFileSize(file.size)}
                </span>
                <div className="flex items-center gap-1 flex-shrink-0">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        disabled={disabled || isBusy}
                        className="opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <ArrowRightLeft className="size-4 text-muted-foreground" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="max-h-64 overflow-y-auto">
                      {formats.map((fmt) => {
                        const info = FORMAT_OPTIONS[fmt];
                        if (!info) return null;
                        return (
                          <DropdownMenuItem
                            key={fmt}
                            onClick={() => onConvertSingle(file.id, fmt)}
                          >
                            {info.label}
                          </DropdownMenuItem>
                        );
                      })}
                    </DropdownMenuContent>
                  </DropdownMenu>
                  {file.status === "done" && file.outputPath && (
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => onOpenFile(file.outputPath!)}
                    >
                      <ExternalLink className="size-4 text-primary" />
                    </Button>
                  )}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    onClick={() => onRemove(file.id)}
                    disabled={disabled}
                    className="opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      </ScrollArea>
    </div>
  );
}
