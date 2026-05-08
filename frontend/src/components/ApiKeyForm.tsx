import { useState } from "react";

interface Props {
  currentMaskedKey: string;
  onSave: (key: string) => Promise<void>;
}

export function ApiKeyForm({ currentMaskedKey, onSave }: Props) {
  const [apiKey, setApiKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSaving(true);
    try {
      await onSave(apiKey);
      setApiKey("");
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save API key");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6 mb-4">
      <h3 className="text-sm font-semibold text-gray-800 mb-4">API Key</h3>
      {currentMaskedKey && (
        <p className="text-sm text-gray-500 mb-3">
          Current: <span className="font-mono">{currentMaskedKey}</span>
        </p>
      )}
      <form onSubmit={handleSubmit} className="space-y-3">
        <input
          data-testid="apikey-input"
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="Enter new API key"
          className="w-full border border-gray-300 rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        {error && <p className="text-sm text-red-600">{error}</p>}
        {success && (
          <p className="text-sm text-green-600">API key updated successfully</p>
        )}
        <button
          data-testid="apikey-submit-button"
          type="submit"
          disabled={apiKey.length === 0 || saving}
          className="px-4 py-2 bg-blue-600 text-white rounded text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {saving ? "Saving..." : "Save API Key"}
        </button>
      </form>
    </div>
  );
}
