import React, { createContext, useContext, useState } from "react";

const SESSION_KEY = "llm_auth";

interface AuthContextValue {
  creds: string | null;
  login: (creds: string) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue>(null!);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [creds, setCreds] = useState<string | null>(
    () => sessionStorage.getItem(SESSION_KEY)
  );

  function login(newCreds: string) {
    sessionStorage.setItem(SESSION_KEY, newCreds);
    setCreds(newCreds);
  }

  function logout() {
    sessionStorage.removeItem(SESSION_KEY);
    setCreds(null);
  }

  return (
    <AuthContext.Provider value={{ creds, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  return useContext(AuthContext);
}
