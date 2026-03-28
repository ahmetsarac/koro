import type { NextConfig } from "next";

import path from "path";
import dotenv from "dotenv";

// repo root .env
dotenv.config({
  path: path.resolve(process.cwd(), "../../.env"),
});

const nextConfig: NextConfig = {
  output: "standalone",
};

export default nextConfig;
