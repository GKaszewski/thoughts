"use client";

import { createContext, useContext, useState, ReactNode } from "react";
import Cookies from "js-cookie";

interface AuthContextType {
  token: string | null;
  setToken: (token: string | null) => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setTokenState] = useState<string | null>(() => {
    return Cookies.get("auth_token") || null;
  });

  const setToken = (newToken: string | null) => {
    setTokenState(newToken);
    if (newToken) {
      Cookies.set("auth_token", newToken, { expires: 7, secure: true });
    } else {
      Cookies.remove("auth_token");
    }
  };

  return (
    <AuthContext.Provider value={{ token, setToken }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
