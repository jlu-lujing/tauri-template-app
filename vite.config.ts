import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],

  // Prevent vite from obscuring Tauri CLI output
  clearScreen: false,

  // Tauri expects a fixed port; fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // Tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },

  // Tauri expects build output at ../dist (but since files are at root now, dist is relative)
  build: {
    outDir: 'dist',
    target: 'esnext',
    minify: false, // Tauri uses rustup for minification
    rollupOptions: {
      output: {
        manualChunks: undefined,
      },
    },
  },
})
