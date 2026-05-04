export const API_ENDPOINTS = {
	STORE: {
		BASE: '/store',
		MEDIA: '/store/api/media',
		SLIDESHOWS: '/store/api/slideshows',
		HEALTH: '/store/api/health'
	},
	PLAYBACK: {
		BASE: '/playback',
		API: '/playback/api',
		STREAM: '/playback/stream'
	}
} as const;
