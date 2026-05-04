import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig(({ mode }) => ({
	plugins: [sveltekit()],
	resolve: {
		alias: {
			'@proto': path.resolve(__dirname, '../proto'),
			$lib: path.resolve(__dirname, 'src/lib')
		}
	},
	server: {
		port: Number(process.env.VITE_DEV_PORT) || 7381,
		host: true,
		...(mode === 'development' && {
			proxy: {
				'/store': {
					target: process.env.VITE_STORE_URL || 'http://localhost:61532',
					changeOrigin: true,
					secure: false,
					rewrite: (path) => path.replace(/^\/store/, '')
				},
				'/playback': {
					target: process.env.VITE_PLAYBACK_SERVER_URL || 'http://localhost:9247',
					changeOrigin: true,
					secure: false,
					rewrite: (path) => path.replace(/^\/playback/, '')
				}
			}
		})
	},
	build: {
		minify: 'esbuild',
		sourcemap: false,
		target: 'es2015',
		rollupOptions: {
			output: {
				manualChunks(id) {
					if (id.includes('node_modules')) {
						return 'vendor';
					}
				}
			}
		},
		chunkSizeWarningLimit: 1000
	},
	esbuild: {
		drop: mode === 'production' ? ['console', 'debugger'] : []
	}
}));
