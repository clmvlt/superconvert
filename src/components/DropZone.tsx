import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import { Upload, Plus, FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getAllSupportedExtensions, getFileCategory } from "@/types/conversion";
import type { FileCategory } from "@/types/conversion";

interface DropZoneProps {
  onFilesAdded: (paths: string[]) => void;
  onFolderAdded: () => void;
  hasFiles: boolean;
  disabled: boolean;
  presentCategories: FileCategory[];
}

const ALL_EXTENSIONS = getAllSupportedExtensions();

export default function DropZone({ onFilesAdded, onFolderAdded, hasFiles, disabled }: DropZoneProps) {
  const { t } = useTranslation();
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();

    const unlistenDrop = appWindow.onDragDropEvent((event) => {
      if (disabled) return;
      if (event.payload.type === "over") {
        setIsDragging(true);
      } else if (event.payload.type === "drop") {
        setIsDragging(false);
        const paths = event.payload.paths.filter((p) => {
          const ext = p.split(".").pop()?.toLowerCase() ?? "";
          return getFileCategory(ext) !== null;
        });
        if (paths.length > 0) {
          onFilesAdded(paths);
        }
      } else if (event.payload.type === "leave") {
        setIsDragging(false);
      }
    });

    return () => {
      unlistenDrop.then((fn) => fn());
    };
  }, [onFilesAdded, disabled]);

  const handleBrowse = async () => {
    if (disabled) return;
    const result = await open({
      multiple: true,
      filters: [
        { name: t("dropzone.filterAllSupported"), extensions: ALL_EXTENSIONS },
        { name: t("dropzone.filterImages"), extensions: ["png", "jpg", "jpeg", "gif", "bmp", "ico", "tif", "tiff", "webp", "avif", "svg", "tga", "qoi", "hdr", "ppm", "pgm", "pbm", "exr", "psd", "heif", "heic", "jxl", "jp2", "j2k", "dds", "cr2", "nef", "arw", "dng", "orf", "rw2"] },
        { name: t("dropzone.filterAudio"), extensions: ["mp3", "wav", "flac", "ogg", "aac", "aiff", "aif", "m4a", "alac", "opus", "wma", "ac3", "dts"] },
        { name: t("dropzone.filterVideo"), extensions: ["mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "mpeg", "mpg", "ts", "3gp", "m4v", "vob"] },
        { name: t("dropzone.filterDocuments"), extensions: ["pdf"] },
        { name: t("dropzone.filterTextDocuments"), extensions: ["docx", "odt", "txt", "rtf", "epub"] },
        { name: t("dropzone.filterSpreadsheets"), extensions: ["xlsx", "xls", "ods", "csv"] },
        { name: t("dropzone.filterPresentations"), extensions: ["pptx", "odp"] },
        { name: t("dropzone.filterData"), extensions: ["json", "yaml", "yml", "toml", "xml", "md", "html", "htm"] },
        { name: t("dropzone.filterArchives"), extensions: ["zip", "tar", "gz", "tgz", "bz2", "xz", "7z", "rar", "zst"] },
      ],
    });
    if (result && result.length > 0) {
      onFilesAdded(result);
    }
  };

  const handleBrowseFolder = () => {
    if (disabled) return;
    onFolderAdded();
  };

  if (hasFiles) {
    return (
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleBrowse}
          disabled={disabled}
          className="flex-1 border-dashed border-border/60 hover:border-border text-muted-foreground hover:text-foreground"
        >
          <Plus className="size-3.5" />
          {t("dropzone.addFiles")}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={handleBrowseFolder}
          disabled={disabled}
          className="border-dashed border-border/60 hover:border-border text-muted-foreground hover:text-foreground"
        >
          <FolderOpen className="size-3.5" />
          {t("dropzone.addFolder")}
        </Button>
      </div>
    );
  }

  return (
    <div
      onClick={handleBrowse}
      className={`
        flex-1 flex flex-col items-center justify-center gap-5 rounded-2xl border-2 border-dashed transition-all duration-300 cursor-pointer
        ${isDragging
          ? "border-primary/60 bg-primary/5 scale-[1.01]"
          : "border-border/40 hover:border-muted-foreground/30 bg-card/30 hover:bg-card/50"
        }
        ${disabled ? "opacity-50 cursor-not-allowed" : ""}
      `}
    >
      <div className={`p-4 rounded-2xl transition-colors duration-300 ${isDragging ? "bg-primary/10" : "bg-muted/50"}`}>
        <Upload className={`size-8 stroke-[1.5] transition-colors duration-300 ${isDragging ? "text-primary" : "text-muted-foreground/50"}`} />
      </div>
      <div className="text-center space-y-1.5">
        <p className="text-sm font-medium text-foreground/80">
          {isDragging ? t("dropzone.dropHere") : t("dropzone.dragAndDrop")}
        </p>
        <p className="text-xs text-muted-foreground/50">
          {t("dropzone.orClickToBrowse")}
        </p>
      </div>
      <Button
        variant="ghost"
        size="sm"
        onClick={(e) => {
          e.stopPropagation();
          handleBrowseFolder();
        }}
        className="text-xs text-muted-foreground/50 hover:text-muted-foreground"
      >
        <FolderOpen className="size-3.5" />
        {t("dropzone.orSelectFolder")}
      </Button>
    </div>
  );
}
