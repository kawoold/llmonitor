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
import type { CacheStats } from "../types/api";

interface Props {
  data: CacheStats;
}

export function CacheBarChart({ data }: Props) {
  const chartData = [
    {
      name: "Cache Tokens",
      read: data.cache_read_tokens,
      creation: data.cache_creation_tokens,
    },
  ];

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="name" tick={{ fontSize: 11 }} />
        <YAxis tick={{ fontSize: 11 }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="read" name="Read" fill="#2563eb" />
        <Bar dataKey="creation" name="Creation" fill="#9333ea" />
      </BarChart>
    </ResponsiveContainer>
  );
}
