import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";


const host = process.env.TAURI_DEV_HOST;


export default defineConfig(async () => ({
  plugins: [vue(), tailwindcss()],

  define: {
    __SENTRY_RELEASE__: JSON.stringify(process.env.SENTRY_RELEASE || 'development'),
    __SENTRY_ENVIRONMENT__: JSON.stringify(process.env.SENTRY_ENVIRONMENT || 'development'),
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
}));
