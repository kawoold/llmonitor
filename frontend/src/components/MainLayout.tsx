import { useState } from "react";
import { useAuth } from "../context/AuthContext";
import { NavTabs } from "./NavTabs";
import { Dashboard } from "./Dashboard";
import { CachePanel } from "./CachePanel";
import { ConfigPanel } from "./ConfigPanel";
import PromptTab from "./PromptTab";
import type { Tab } from "./NavTabs";

export function MainLayout() {
  const { logout } = useAuth();
  const [activeTab, setActiveTab] = useState<Tab>("dashboard");

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-3 flex items-center justify-between">
        <span className="font-semibold text-gray-800">llmonitor</span>
        <button
          onClick={logout}
          className="text-sm text-gray-500 hover:text-gray-700"
        >
          Sign out
        </button>
      </header>
      <main className={activeTab === "prompt" ? "px-6 py-6" : "max-w-5xl mx-auto px-6 py-6"}>
        <NavTabs activeTab={activeTab} onTabChange={setActiveTab} />
        {activeTab === "dashboard" && <Dashboard />}
        {activeTab === "cache" && <CachePanel />}
        {activeTab === "config" && <ConfigPanel />}
        {activeTab === "prompt" && <PromptTab />}
      </main>
    </div>
  );
}
