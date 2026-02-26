import { Check, X, Loader2, Circle, FileSearch, FolderOpen, Trash2, ArrowRightLeft } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ConversionFile, FileCategory } from "@/types/conversion";
import { getOutputFormats, FORMAT_OPTIONS, CATEGORY_COLORS } from "@/types/conversion";
import { formatFileSize } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
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
  onOpenFolder: (path: string) => void;
  onConvertSingle: (fileId: string, format: string) => void;
  disabled: boolean;
}

function StatusIcon({ status }: { status: ConversionFile["status"] }) {
  switch (status) {
    case "done":
      return (
        <div className="size-5 rounded-full bg-green-500/15 flex items-center justify-center">
          <Check className="size-3 text-green-400" />
        </div>
      );
    case "error":
      return (
        <div className="size-5 rounded-full bg-red-500/15 flex items-center justify-center">
          <X className="size-3 text-red-400" />
        </div>
      );
    case "converting":
      return <Loader2 className="size-4 text-blue-400 animate-spin" />;
    default:
      return <Circle className="size-2.5 text-muted-foreground/20" />;
  }
}

export default function FileList({ files, outputFormats, onRemove, onOpenFile, onOpenFolder, onConvertSingle, disabled }: FileListProps) {
  const { t } = useTranslation();

  if (files.length === 0) return null;

  return (
    <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
      <div className="flex items-center justify-between mb-2 px-1 flex-shrink-0">
        <span className="text-xs font-medium text-muted-foreground/50">
          {t("fileList.fileCount", { count: files.length })}
        </span>
      </div>
      <ScrollArea className="flex-1 min-h-0">
        <div className="space-y-1 pr-3">
          {files.map((file) => {
            const targetFormat = outputFormats[file.category] ?? "?";
            const formats = getOutputFormats(file.category);
            const isBusy = file.status === "converting";
            return (
              <div
                key={file.id}
                className="flex items-center gap-3 px-3.5 py-2.5 rounded-lg group hover:bg-card/60 transition-all duration-150"
              >
                <div className="w-5 flex-shrink-0 flex justify-center">
                  <StatusIcon status={file.status} />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-[13px] text-foreground/90 truncate font-medium">
                      {file.name}
                    </span>
                    <div className="flex items-center gap-1 flex-shrink-0">
                      <span className={`text-[10px] font-semibold px-1.5 py-px rounded uppercase ${CATEGORY_COLORS[file.category]}`}>
                        {file.extension}
                      </span>
                      <span className="text-[10px] text-muted-foreground/30 px-0.5">
                        &rarr;
                      </span>
                      <span className="text-[10px] font-semibold px-1.5 py-px rounded uppercase bg-primary/10 text-primary">
                        {targetFormat}
                      </span>
                    </div>
                  </div>
                  {file.status === "converting" && (
                    <div className="mt-1.5 h-1 bg-muted/50 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-blue-500/80 rounded-full transition-all duration-300 ease-out"
                        style={{ width: `${Math.round(file.progress * 100)}%` }}
                      />
                    </div>
                  )}
                  {file.status === "error" && file.error && (
                    <p className="text-[11px] text-red-400/80 mt-1 truncate">
                      {file.error}
                    </p>
                  )}
                </div>
                <span className="text-[11px] text-muted-foreground/40 flex-shrink-0 tabular-nums">
                  {formatFileSize(file.size)}
                </span>
                <div className="flex items-center gap-0.5 flex-shrink-0">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        disabled={disabled || isBusy}
                        className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground/50 hover:text-foreground"
                      >
                        <ArrowRightLeft className="size-3" />
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
                    <>
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        onClick={() => onOpenFile(file.outputPath!)}
                        title={t("fileList.openFile")}
                        className="text-primary/70 hover:text-primary"
                      >
                        <FileSearch className="size-3" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        onClick={() => onOpenFolder(file.outputPath!)}
                        title={t("fileList.openFolder")}
                        className="text-muted-foreground/50 hover:text-foreground"
                      >
                        <FolderOpen className="size-3" />
                      </Button>
                    </>
                  )}
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    onClick={() => onRemove(file.id)}
                    disabled={disabled}
                    className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground/30 hover:text-red-400"
                  >
                    <Trash2 className="size-3" />
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
