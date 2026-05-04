import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/**
 * Simple configuration loader for SvelteKit build process
 * Reads environment variables directly without async operations
 */
const getConfig = () => {
	// Read from environment variables
	const playbackWsUrl = process.env.VITE_PLAYBACK_WS_URL || 'ws://localhost:61532';
	const playbackServerUrl = process.env.VITE_PLAYBACK_SERVER_URL || 'http://localhost:9247';
	const storeUrl = process.env.VITE_STORE_URL || 'http://localhost:61532';

	try {
		return {
			playbackWsUrl,
			playbackServerUrl,
			storeUrl,
			storeHost: new URL(storeUrl).hostname,
			playbackHost: new URL(playbackWsUrl).hostname,
			playbackServerHost: new URL(playbackServerUrl).hostname
		};
	} catch {
		return {
			playbackWsUrl,
			playbackServerUrl,
			storeUrl,
			storeHost: 'localhost',
			playbackHost: 'localhost',
			playbackServerHost: 'localhost'
		};
	}
};

// Dynamic CSP configuration
const cspConfig = getConfig();

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Enable TypeScript compilation for Svelte files
	compilerOptions: {
		dev: process.env.NODE_ENV !== 'production',
		runes: true // Enable Svelte 5 runes ($state, $derived, etc.)
	},
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	// NOTE: Since Svelte 5, script: true is not needed for TypeScript anymore
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			// Static adapter configuration
			pages: 'build',
			assets: 'build',
			fallback: 'index.html',
			precompress: false
		}),

		// Path aliases
		alias: {
			$lib: './src/lib',
			$config: './src/config/index.ts'
		},

		// Modern kit configuration
		csp: {
			mode: 'auto',
			directives: {
				'script-src': ['self'],
				'style-src': ['self', 'unsafe-inline'],
				'default-src': ['self'],
				'connect-src': [
					'self',
					cspConfig.playbackWsUrl,
					cspConfig.playbackServerUrl,
					cspConfig.storeUrl,
					`http://${cspConfig.storeHost}`,
					`https://${cspConfig.storeHost}`,
					`http://${cspConfig.playbackServerHost}`,
					`https://${cspConfig.playbackServerHost}`,
					`ws://${cspConfig.playbackHost}`,
					`wss://${cspConfig.playbackHost}`
				]
			}
		}
	}
};

export default config;
