import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { UsageDataPoint } from "../types/api";

interface Props {
  data: UsageDataPoint[];
}

export function UsageLineChart({ data }: Props) {
  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-72 text-gray-400 text-sm">
        No data for this window
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={data}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="bucket"
          tick={{ fontSize: 11 }}
          tickFormatter={(v: string) => v.slice(5, 16)}
        />
        <YAxis tick={{ fontSize: 11 }} />
        <Tooltip />
        <Legend />
        <Line
          type="monotone"
          dataKey="total_tokens"
          name="Total"
          stroke="#2563eb"
          dot={false}
        />
        <Line
          type="monotone"
          dataKey="prompt_tokens"
          name="Prompt"
          stroke="#16a34a"
          dot={false}
        />
        <Line
          type="monotone"
          dataKey="completion_tokens"
          name="Completion"
          stroke="#dc2626"
          dot={false}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
