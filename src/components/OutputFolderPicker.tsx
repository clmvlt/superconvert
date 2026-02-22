import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";

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
    <Button
      variant="outline"
      onClick={handlePick}
      disabled={disabled}
      className="w-full justify-start text-left h-auto py-2"
    >
      <FolderOpen className="size-4 shrink-0" />
      {folder ? (
        <span className="truncate text-foreground">{folder}</span>
      ) : (
        <span className="text-muted-foreground">Same as source (default)</span>
      )}
    </Button>
  );
}
