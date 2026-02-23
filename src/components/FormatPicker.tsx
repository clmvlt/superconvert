import { FORMAT_OPTIONS, CATEGORY_LABELS, CATEGORY_COLORS } from "@/types/conversion";
import type { FileCategory } from "@/types/conversion";
import { Button } from "@/components/ui/button";

interface FormatPickerProps {
  category: FileCategory;
  showLabel: boolean;
  formats: string[];
  selected: string;
  onChange: (format: string) => void;
  disabled: boolean;
}

export default function FormatPicker({ category, showLabel, formats, selected, onChange, disabled }: FormatPickerProps) {
  return (
    <div>
      {showLabel && (
        <div className="flex items-center gap-2 mb-2">
          <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${CATEGORY_COLORS[category]}`}>
            {CATEGORY_LABELS[category]}
          </span>
        </div>
      )}
      <div className="grid grid-cols-4 gap-1.5">
        {formats.map((fmt) => {
          const info = FORMAT_OPTIONS[fmt];
          const isSelected = selected === fmt;
          if (!info) return null;
          return (
            <Button
              key={fmt}
              variant={isSelected ? "default" : "secondary"}
              size="sm"
              onClick={() => onChange(fmt)}
              disabled={disabled}
              className={
                isSelected
                  ? `${info.color} text-white hover:opacity-90 shadow-md`
                  : ""
              }
            >
              {info.label}
            </Button>
          );
        })}
      </div>
    </div>
  );
}
