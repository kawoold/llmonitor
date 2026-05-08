import { useState } from "react";
import { useAuth } from "../context/AuthContext";

interface Props {
  onSave: (username: string, password: string) => Promise<void>;
}

export function CredentialsForm({ onSave }: Props) {
  const { logout } = useAuth();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSaving(true);
    try {
      await onSave(username, password);
      setSuccess(true);
      setTimeout(() => {
        logout();
        window.location.reload();
      }, 1500);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to update credentials"
      );
      setSaving(false);
    }
  }

  const canSubmit = username.length > 0 && password.length > 0 && !saving;

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-6">
      <h3 className="text-sm font-semibold text-gray-800 mb-4">
        Admin Credentials
      </h3>
      <form onSubmit={handleSubmit} className="space-y-3">
        <div>
          <label className="block text-sm text-gray-600 mb-1">
            New Username
          </label>
          <input
            data-testid="credentials-username-input"
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="w-full border border-gray-300 rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            autoComplete="username"
          />
        </div>
        <div>
          <label className="block text-sm text-gray-600 mb-1">
            New Password
          </label>
          <input
            data-testid="credentials-password-input"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="w-full border border-gray-300 rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            autoComplete="new-password"
          />
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
        {success && (
          <p className="text-sm text-green-600">
            Credentials updated. Logging out...
          </p>
        )}
        <button
          data-testid="credentials-submit-button"
          type="submit"
          disabled={!canSubmit}
          className="px-4 py-2 bg-blue-600 text-white rounded text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {saving ? "Saving..." : "Update Credentials"}
        </button>
      </form>
    </div>
  );
}
