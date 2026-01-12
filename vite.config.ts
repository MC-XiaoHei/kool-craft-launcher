import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import tailwindcss from "@tailwindcss/vite"
import tsconfigPaths from "vite-tsconfig-paths"
import vueRouter from "unplugin-vue-router/vite"
import { resolve } from "path"

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vueRouter({
      routesFolder: "src/pages",
      extensions: [".vue"],
      dts: "src/typed-router.d.ts",
    }),
    vue(),
    tailwindcss(),
    tsconfigPaths(),
  ],

  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}))
