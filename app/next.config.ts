import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  experimental: {
    optimizePackageImports: ["@chakra-ui/react"],
  },

  // ✅ MUST be top-level - CORRECT
  // ❌ Remove http:// or https:// - just the hostname
  allowedDevOrigins: [
    "localhost:3000",
    "*.ngrok-free.app",     // All ngrok tunnels
    "*.trycloudflare.com",  // Cloudflare tunnels
  ],


  // ✅ ADD THIS (fixes your <Image> external URL issue)
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "encrypted-tbn0.gstatic.com",
      },
    ],
  },


  output: "standalone",

  env: {
    NEXT_PUBLIC_GOOGLE_CLIENT_ID: process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID,
    NEXT_PUBLIC_SERVICE_API_URL: process.env.NEXT_PUBLIC_SERVICE_API_URL,
    NEXT_PUBLIC_GRPC_API_URL: process.env.NEXT_PUBLIC_GRPC_API_URL,
    NEXT_PUBLIC_WS_API_URL: process.env.NEXT_PUBLIC_WS_API_URL,
  },
};

export default nextConfig;