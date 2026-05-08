import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { apiFetch, UnauthorizedError } from "../lib/apiFetch";
import { WindowSelector } from "./WindowSelector";
import { CacheStatCards } from "./CacheStatCards";
import { CacheBarChart } from "./CacheBarChart";
import type { TimeWindow, CacheStats } from "../types/api";

export function CachePanel() {
  const { creds, logout } = useAuth();
  const [window, setWindow] = useState<TimeWindow>("24h");
  const [cacheData, setCacheData] = useState<CacheStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function fetchData() {
      setLoading(true);
      setError(null);
      try {
        const res = await apiFetch(`/api/stats/cache?window=${window}`, creds!);
        if (cancelled) return;
        const data = await res.json() as CacheStats;
        setCacheData(data);
      } catch (err) {
        if (cancelled) return;
        if (err instanceof UnauthorizedError) {
          logout();
          return;
        }
        setError(err instanceof Error ? err.message : "Failed to load cache data");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void fetchData();
    return () => { cancelled = true; };
  }, [window, creds, logout]);

  return (
    <div>
      <h2 className="text-lg font-semibold text-gray-800 mb-4">Cache</h2>
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
      {!loading && !error && cacheData && (
        cacheData.total_requests === 0 ? (
          <div className="text-gray-400 text-sm py-12 text-center">
            No cache activity in this window
          </div>
        ) : (
          <>
            <CacheStatCards data={cacheData} />
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <h3 className="text-sm font-medium text-gray-700 mb-3">
                Cache Token Breakdown
              </h3>
              <CacheBarChart data={cacheData} />
            </div>
          </>
        )
      )}
    </div>
  );
}
