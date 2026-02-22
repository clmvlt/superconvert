import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import { Upload, Plus, FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getAllSupportedExtensions, getFileCategory, CATEGORY_LABELS, CATEGORY_COLORS } from "@/types/conversion";
import type { FileCategory } from "@/types/conversion";

interface DropZoneProps {
  onFilesAdded: (paths: string[]) => void;
  onFolderAdded: () => void;
  hasFiles: boolean;
  disabled: boolean;
  detectedCategory: FileCategory | null;
}

const ALL_EXTENSIONS = getAllSupportedExtensions();

export default function DropZone({ onFilesAdded, onFolderAdded, hasFiles, disabled, detectedCategory }: DropZoneProps) {
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
        {
          name: "All supported",
          extensions: ALL_EXTENSIONS,
        },
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "gif", "bmp", "ico", "tif", "tiff", "webp", "avif", "svg", "tga", "qoi"],
        },
        {
          name: "Audio",
          extensions: ["mp3", "wav", "flac", "ogg", "aac", "aiff", "m4a"],
        },
        {
          name: "Documents",
          extensions: ["pdf"],
        },
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
          onClick={handleBrowse}
          disabled={disabled}
          className="flex-1"
        >
          <Plus className="size-4" />
          Add files
        </Button>
        <Button
          variant="outline"
          onClick={handleBrowseFolder}
          disabled={disabled}
        >
          <FolderOpen className="size-4" />
          Add folder
        </Button>
        {detectedCategory && (
          <span className={`text-xs font-medium px-2.5 py-1 rounded-full ${CATEGORY_COLORS[detectedCategory]}`}>
            {CATEGORY_LABELS[detectedCategory]}
          </span>
        )}
      </div>
    );
  }

  return (
    <div
      onClick={handleBrowse}
      className={`
        flex flex-col items-center justify-center gap-4 p-12 rounded-xl border-2 border-dashed transition-all duration-200 cursor-pointer
        ${isDragging
          ? "border-primary bg-primary/5"
          : "border-border hover:border-muted-foreground/50 bg-card/50 hover:bg-card"
        }
        ${disabled ? "opacity-50 cursor-not-allowed" : ""}
      `}
    >
      <div className="text-muted-foreground">
        <Upload className="size-12 stroke-[1.5]" />
      </div>
      <div className="text-center">
        <p className="text-lg font-medium text-foreground">
          {isDragging ? "Drop files here" : "Drag & drop files here"}
        </p>
        <p className="text-sm text-muted-foreground mt-1">
          or click to browse
        </p>
      </div>
      <Button
        variant="ghost"
        size="sm"
        onClick={(e) => {
          e.stopPropagation();
          handleBrowseFolder();
        }}
        className="text-primary"
      >
        <FolderOpen className="size-4" />
        or select a folder
      </Button>
      <div className="flex flex-wrap justify-center gap-x-3 gap-y-1 text-xs text-muted-foreground/60">
        <span>Images: PNG, JPG, WebP, GIF, BMP, TIFF, AVIF, ICO, SVG</span>
        <span>Audio: MP3, WAV, FLAC, OGG, AAC</span>
        <span>Documents: PDF</span>
      </div>
    </div>
  );
}
