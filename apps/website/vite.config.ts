import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  base: "./",
  plugins: [react()],
  build: { target: ["es2020", "chrome90", "edge90", "firefox90", "safari14"] },
});
