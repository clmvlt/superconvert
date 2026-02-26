import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-react";

interface OutputFolderPickerProps {
  folder: string;
  onChange: (folder: string) => void;
  disabled: boolean;
}

export default function OutputFolderPicker({
  folder,
  onChange,
  disabled,
}: OutputFolderPickerProps) {
  const { t } = useTranslation();

  const handlePick = async () => {
    const result = await open({
      directory: true,
      multiple: false,
    });
    if (result) {
      onChange(result);
    }
  };

  return (
    <button
      onClick={handlePick}
      disabled={disabled}
      className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg border border-border/40 bg-background/50 hover:bg-background/80 hover:border-border/60 transition-all duration-150 text-left cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
    >
      <FolderOpen className="size-3.5 shrink-0 text-muted-foreground/50" />
      {folder ? (
        <span className="truncate text-xs text-foreground/80">{folder}</span>
      ) : (
        <span className="text-xs text-muted-foreground/40">{t("outputFolder.placeholder")}</span>
      )}
    </button>
  );
}
