import type { CacheStats } from "../types/api";

interface Props {
  data: CacheStats;
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <p className="text-sm text-gray-500">{label}</p>
      <p className="text-2xl font-semibold text-gray-800 mt-1">{value}</p>
    </div>
  );
}

export function CacheStatCards({ data }: Props) {
  return (
    <div className="grid grid-cols-2 gap-4 mb-6 sm:grid-cols-4">
      <StatCard
        label="Hit Rate"
        value={`${(data.cache_hit_rate * 100).toFixed(1)}%`}
      />
      <StatCard
        label="Read Tokens"
        value={data.cache_read_tokens.toLocaleString()}
      />
      <StatCard
        label="Creation Tokens"
        value={data.cache_creation_tokens.toLocaleString()}
      />
      <StatCard
        label="Cached Requests"
        value={data.total_requests.toLocaleString()}
      />
    </div>
  );
}
