import { useTranslation } from "react-i18next";
import { FORMAT_OPTIONS, CATEGORY_COLORS } from "@/types/conversion";
import type { FileCategory } from "@/types/conversion";

interface FormatPickerProps {
  category: FileCategory;
  showLabel: boolean;
  formats: string[];
  selected: string;
  onChange: (format: string) => void;
  disabled: boolean;
}

export default function FormatPicker({ category, showLabel, formats, selected, onChange, disabled }: FormatPickerProps) {
  const { t } = useTranslation();

  return (
    <div>
      {showLabel && (
        <div className="mb-2">
          <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full uppercase tracking-wide ${CATEGORY_COLORS[category]}`}>
            {t(`categories.${category}`)}
          </span>
        </div>
      )}
      <div className="flex flex-wrap gap-1">
        {formats.map((fmt) => {
          const info = FORMAT_OPTIONS[fmt];
          const isSelected = selected === fmt;
          if (!info) return null;
          return (
            <button
              key={fmt}
              onClick={() => onChange(fmt)}
              disabled={disabled}
              className={`
                px-2.5 py-1 rounded-md text-[11px] font-semibold uppercase transition-all duration-150 cursor-pointer
                disabled:opacity-40 disabled:cursor-not-allowed
                ${isSelected
                  ? `${info.color} text-white shadow-sm`
                  : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground"
                }
              `}
            >
              {info.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}
