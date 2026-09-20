interface RangeControlProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  format?: (value: number) => string;
}

export function RangeControl({ label, value, min, max, step, onChange, format }: RangeControlProps) {
  const progress = ((value - min) / (max - min)) * 100;
  return (
    <label className="range-control">
      <span className="range-label">
        <span>{label}</span>
        <output>{format ? format(value) : value.toFixed(2)}</output>
      </span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        style={{ "--range-progress": `${progress}%` } as React.CSSProperties}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </label>
  );
}
