import type { StatsSummary } from "../types/api";

interface Props {
  summary: StatsSummary;
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <p className="text-sm text-gray-500">{label}</p>
      <p className="text-2xl font-semibold text-gray-800 mt-1">
        {value.toLocaleString()}
      </p>
    </div>
  );
}

export function SummaryCards({ summary }: Props) {
  return (
    <div className="grid grid-cols-2 gap-4 mb-6 sm:grid-cols-4">
      <StatCard label="Total Requests" value={summary.total_requests} />
      <StatCard label="Total Tokens" value={summary.total_tokens} />
      <StatCard label="Prompt Tokens" value={summary.prompt_tokens} />
      <StatCard label="Completion Tokens" value={summary.completion_tokens} />
    </div>
  );
}
