import { API_ENDPOINTS } from '$lib/api/config';

export const getMediaUrl = (filename: string): string => {
	return `${API_ENDPOINTS.STORE.MEDIA}/${filename}`;
};
