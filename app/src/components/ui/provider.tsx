"use client";

import { ChakraProvider, defaultSystem } from "@chakra-ui/react";
import { GoogleOAuthProvider } from "@react-oauth/google";
import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { ColorModeProvider, type ColorModeProviderProps } from "./color-mode";
import { Toaster } from "./toaster";

export function Provider({ children, ...props }: ColorModeProviderProps & { children: React.ReactNode }) {
  const [client] = useState(() => new QueryClient());

  return (
    <QueryClientProvider client={client}>
      <GoogleOAuthProvider
        clientId={process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID ?? ""}
      >
        <ChakraProvider value={defaultSystem}>
          <ColorModeProvider {...props}>
            {children}   {/* ✅ CRITICAL FIX */}
          </ColorModeProvider>
          <Toaster />
        </ChakraProvider>
      </GoogleOAuthProvider>
    </QueryClientProvider>
  );
}