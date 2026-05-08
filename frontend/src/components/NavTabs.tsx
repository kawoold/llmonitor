export type Tab = "dashboard" | "cache" | "config" | "prompt";

interface Props {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
}

const TABS: { id: Tab; label: string; testId: string }[] = [
  { id: "dashboard", label: "Dashboard", testId: "nav-dashboard" },
  { id: "cache", label: "Cache", testId: "nav-cache" },
  { id: "config", label: "Config", testId: "nav-config" },
  { id: "prompt", label: "Prompt", testId: "nav-prompt" },
];

export function NavTabs({ activeTab, onTabChange }: Props) {
  return (
    <nav className="flex gap-1 border-b border-gray-200 mb-6">
      {TABS.map(({ id, label, testId }) => (
        <button
          key={id}
          data-testid={testId}
          onClick={() => onTabChange(id)}
          className={
            activeTab === id
              ? "px-4 py-2 text-sm font-medium border-b-2 border-blue-600 text-blue-600"
              : "px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-800"
          }
        >
          {label}
        </button>
      ))}
    </nav>
  );
}
