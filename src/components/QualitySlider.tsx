import { LOSSY_FORMATS } from "@/types/conversion";
import { Slider } from "@/components/ui/slider";

interface QualitySliderProps {
  format: string;
  quality: number;
  onChange: (quality: number) => void;
  disabled: boolean;
}

export default function QualitySlider({
  format,
  quality,
  onChange,
  disabled,
}: QualitySliderProps) {
  if (!LOSSY_FORMATS.includes(format)) return null;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-card-foreground">
          Quality
        </label>
        <span className="text-sm text-muted-foreground tabular-nums">
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
      <div className="flex justify-between text-xs text-muted-foreground/60">
        <span>Low</span>
        <span>High</span>
      </div>
    </div>
  );
}
