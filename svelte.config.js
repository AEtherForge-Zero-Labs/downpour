import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
export default {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    csp: { mode: 'hash', directives: {
      'default-src': ['self'], 'script-src': ['self'], 'style-src': ['self', 'unsafe-inline'],
      'img-src': ['self', 'data:'], 'connect-src': ['self', 'ipc:', 'http://ipc.localhost'],
      'object-src': ['none'], 'base-uri': ['self']
    } }
  }
};
