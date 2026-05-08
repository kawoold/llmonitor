import { AuthProvider, useAuth } from "./context/AuthContext";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AuthGate } from "./components/AuthGate";
import { MainLayout } from "./components/MainLayout";

function AppInner() {
  const { creds } = useAuth();
  return creds !== null ? <MainLayout /> : <AuthGate />;
}

export function App() {
  return (
    <AuthProvider>
      <ErrorBoundary>
        <AppInner />
      </ErrorBoundary>
    </AuthProvider>
  );
}
