// contexts/AuthContext.js
"use client";

import { createContext, useContext, useEffect, useState } from 'react';
import cookie from 'js-cookie';
import { jwtDecode } from 'jwt-decode';

// Define the type for your auth context
interface AuthContextType {
  isAdmin: boolean;
  isLoading: boolean;
  token: string | null;
}

// Create context with a default value
const AuthContext = createContext<AuthContextType>({
  isAdmin: false,
  isLoading: true,
  token: null,
});

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isAdmin, setIsAdmin] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [token, setToken] = useState<string | null>(null);

  useEffect(() => {
    const authToken = cookie.get("polymarketAuthToken");

    if (!authToken) {
      setToken(null);
      setIsAdmin(false);
      setIsLoading(false);
      return;
    }

    try {
      const claims: { is_admin: boolean } = jwtDecode(authToken);
      setToken(authToken);
      setIsAdmin(claims.is_admin);
    } catch (err) {
      console.error("Failed to decode token:", err);
      setToken(null);
      setIsAdmin(false);
    } finally {
      setIsLoading(false);
    }
  }, []);

  return (
    <AuthContext.Provider value={{ isAdmin, isLoading, token }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  // No need for the check now since we provided a default value
  return context;
}