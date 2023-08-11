/** @type {import('@vanilla-extract/next-plugin').VanillaExtractPlugin} */
import { createVanillaExtractPlugin } from "@vanilla-extract/next-plugin";
const withVanillaExtract = createVanillaExtractPlugin();

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  modularizeImports: {
    "@mui/icons-material": {
      transform: "@mui/icons-material/{{member}}",
    },
  },
};

if (process.env.NODE_ENV === "development") {
  nextConfig.rewrites = async () => {
    return [
      {
        source: "/api/:path*",
        destination: "http://localhost:8000/api/:path*",
      },
    ];
  };
}

export default withVanillaExtract(nextConfig);
