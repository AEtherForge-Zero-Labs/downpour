import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
export default defineConfig({ plugins: [sveltekit()], clearScreen: false, server: { port: 1420, strictPort: true, host: process.env.TAURI_DEV_HOST || false, watch: { ignored: ['**/src-tauri/**', '**/target/**'] } } });
