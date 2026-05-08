import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { apiFetch, UnauthorizedError } from "../lib/apiFetch";
import { WindowSelector } from "./WindowSelector";
import { SummaryCards } from "./SummaryCards";
import { UsageLineChart } from "./UsageLineChart";
import { ProviderBarChart } from "./ProviderBarChart";
import type {
  TimeWindow,
  StatsSummary,
  UsageDataPoint,
  ProviderSummary,
} from "../types/api";

export function Dashboard() {
  const { creds, logout } = useAuth();
  const [window, setWindow] = useState<TimeWindow>("24h");
  const [summary, setSummary] = useState<StatsSummary | null>(null);
  const [timeseries, setTimeseries] = useState<UsageDataPoint[]>([]);
  const [providers, setProviders] = useState<ProviderSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function fetchData() {
      setLoading(true);
      setError(null);
      try {
        const [summaryRes, usageRes] = await Promise.all([
          apiFetch(`/api/stats/summary?window=${window}`, creds!),
          apiFetch(`/api/stats/usage?window=${window}`, creds!),
        ]);
        if (cancelled) return;
        const [summaryData, usageData] = await Promise.all([
          summaryRes.json() as Promise<StatsSummary>,
          usageRes.json() as Promise<UsageDataPoint[]>,
        ]);
        setSummary(summaryData);
        setTimeseries(usageData);
        setProviders(summaryData.by_provider);
      } catch (err) {
        if (cancelled) return;
        if (err instanceof UnauthorizedError) {
          logout();
          return;
        }
        setError(err instanceof Error ? err.message : "Failed to load data");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void fetchData();
    return () => { cancelled = true; };
  }, [window, creds, logout]);

  return (
    <div>
      <h2 className="text-lg font-semibold text-gray-800 mb-4">Dashboard</h2>
      <WindowSelector value={window} onChange={setWindow} />
      {loading && (
        <div className="flex justify-center py-12">
          <div className="w-8 h-8 border-4 border-blue-600 border-t-transparent rounded-full animate-spin" />
        </div>
      )}
      {error && !loading && (
        <div className="bg-red-50 border border-red-200 text-red-700 rounded p-4 text-sm mb-4">
          {error}
        </div>
      )}
      {!loading && !error && summary && (
        <>
          <SummaryCards summary={summary} />
          <div className="bg-white rounded-lg border border-gray-200 p-4 mb-4">
            <h3 className="text-sm font-medium text-gray-700 mb-3">
              Token Usage Over Time
            </h3>
            <UsageLineChart data={timeseries} />
          </div>
          <div className="bg-white rounded-lg border border-gray-200 p-4">
            <h3 className="text-sm font-medium text-gray-700 mb-3">
              Usage by Provider / Model
            </h3>
            <ProviderBarChart data={providers} />
          </div>
        </>
      )}
    </div>
  );
}
