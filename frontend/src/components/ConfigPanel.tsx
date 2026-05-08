import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { apiFetch, UnauthorizedError } from "../lib/apiFetch";
import { ApiKeyForm } from "./ApiKeyForm";
import { CredentialsForm } from "./CredentialsForm";
import type { ConfigResponse } from "../types/api";

export function ConfigPanel() {
  const { creds, logout } = useAuth();
  const [config, setConfig] = useState<ConfigResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function fetchConfig() {
      setLoading(true);
      setError(null);
      try {
        const res = await apiFetch("/api/config", creds!);
        if (cancelled) return;
        const data = await res.json() as ConfigResponse;
        setConfig(data);
      } catch (err) {
        if (cancelled) return;
        if (err instanceof UnauthorizedError) {
          logout();
          return;
        }
        setError(err instanceof Error ? err.message : "Failed to load config");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void fetchConfig();
    return () => { cancelled = true; };
  }, [creds, logout]);

  async function handleSaveApiKey(key: string) {
    const res = await apiFetch("/api/config", creds!, {
      method: "PUT",
      body: JSON.stringify({ anthropic_api_key: key }),
    });
    if (!res.ok) throw new Error(`Failed to save: HTTP ${res.status}`);
    const updated = await res.json() as ConfigResponse;
    setConfig(updated);
  }

  async function handleSaveCredentials(username: string, password: string) {
    const res = await apiFetch("/api/credentials", creds!, {
      method: "POST",
      body: JSON.stringify({ username, password }),
    });
    if (!res.ok) throw new Error(`Failed to save: HTTP ${res.status}`);
  }

  return (
    <div>
      <h2 className="text-lg font-semibold text-gray-800 mb-4">Config</h2>
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
      {!loading && (
        <>
          <ApiKeyForm
            currentMaskedKey={config?.anthropic_api_key ?? ""}
            onSave={handleSaveApiKey}
          />
          <CredentialsForm onSave={handleSaveCredentials} />
        </>
      )}
    </div>
  );
}
