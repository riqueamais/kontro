import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// A porta e fixa porque o Tauri precisa saber onde procurar o front em
// desenvolvimento; deixar o Vite escolher outra quebraria o devUrl.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "chrome110", minify: "esbuild", sourcemap: false },
});
