import { FORMAT_OPTIONS, FORMAT_DESCRIPTIONS } from "@/types/conversion";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";

interface FormatPickerProps {
  formats: string[];
  selected: string;
  onChange: (format: string) => void;
  disabled: boolean;
}

export default function FormatPicker({ formats, selected, onChange, disabled }: FormatPickerProps) {
  return (
    <div className="grid grid-cols-4 gap-1.5">
      {formats.map((fmt) => {
        const info = FORMAT_OPTIONS[fmt];
        const isSelected = selected === fmt;
        if (!info) return null;
        return (
          <Tooltip key={fmt}>
            <TooltipTrigger asChild>
              <Button
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
            </TooltipTrigger>
            <TooltipContent side="bottom">
              {FORMAT_DESCRIPTIONS[fmt] ?? fmt.toUpperCase()}
            </TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}
