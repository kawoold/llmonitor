import type { TimeWindow } from "../types/api";

interface Props {
  value: TimeWindow;
  onChange: (w: TimeWindow) => void;
}

const WINDOWS: { value: TimeWindow; label: string; testId: string }[] = [
  { value: "1h", label: "1h", testId: "window-selector-1h" },
  { value: "24h", label: "24h", testId: "window-selector-24h" },
  { value: "7d", label: "7d", testId: "window-selector-7d" },
  { value: "30d", label: "30d", testId: "window-selector-30d" },
];

export function WindowSelector({ value, onChange }: Props) {
  return (
    <div className="flex gap-1 mb-4">
      {WINDOWS.map(({ value: w, label, testId }) => (
        <button
          key={w}
          data-testid={testId}
          onClick={() => onChange(w)}
          className={
            value === w
              ? "px-3 py-1 text-sm rounded bg-blue-600 text-white"
              : "px-3 py-1 text-sm rounded bg-gray-100 text-gray-700 hover:bg-gray-200"
          }
        >
          {label}
        </button>
      ))}
    </div>
  );
}
