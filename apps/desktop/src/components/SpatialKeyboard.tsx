const rows = [
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
  ["Z", "X", "C", "V", "B", "N", "M"],
];

export function SpatialKeyboard({ strength }: { strength: number }) {
  return (
    <div className="spatial-visual" aria-label={`Keyboard spatial field at ${Math.round(strength * 100)} percent`}>
      <div className="spatial-axis"><span>L</span><i /><span>R</span></div>
      <div className="keyboard" style={{ "--spatial": strength } as React.CSSProperties}>
        {rows.map((row, rowIndex) => (
          <div className="keyboard-row" key={row[0]} style={{ paddingInlineStart: `${rowIndex * 18}px` }}>
            {row.map((key, index) => {
              const normalized = (index / Math.max(row.length - 1, 1)) * 2 - 1;
              return <kbd key={key} style={{ "--key-position": normalized } as React.CSSProperties}>{key}</kbd>;
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
