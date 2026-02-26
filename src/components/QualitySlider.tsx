import { useTranslation } from "react-i18next";
import { Slider } from "@/components/ui/slider";

interface QualitySliderProps {
  quality: number;
  onChange: (quality: number) => void;
  disabled: boolean;
}

export default function QualitySlider({
  quality,
  onChange,
  disabled,
}: QualitySliderProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-2.5 pt-1 border-t border-border/30">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-muted-foreground/70">
          {t("quality.label")}
        </span>
        <span className="text-xs font-semibold text-foreground/80 tabular-nums">
          {quality}%
        </span>
      </div>
      <Slider
        min={1}
        max={100}
        step={1}
        value={[quality]}
        onValueChange={([val]) => onChange(val)}
        disabled={disabled}
      />
    </div>
  );
}
