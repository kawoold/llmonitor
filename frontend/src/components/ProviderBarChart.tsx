import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { ProviderSummary } from "../types/api";

interface Props {
  data: ProviderSummary[];
}

export function ProviderBarChart({ data }: Props) {
  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-72 text-gray-400 text-sm">
        No data
      </div>
    );
  }

  const chartData = data.map((d) => ({
    name: `${d.provider}/${d.model}`,
    prompt_tokens: d.prompt_tokens,
    completion_tokens: d.completion_tokens,
  }));

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="name" tick={{ fontSize: 11 }} />
        <YAxis tick={{ fontSize: 11 }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="prompt_tokens" name="Prompt" fill="#16a34a" />
        <Bar dataKey="completion_tokens" name="Completion" fill="#dc2626" />
      </BarChart>
    </ResponsiveContainer>
  );
}
