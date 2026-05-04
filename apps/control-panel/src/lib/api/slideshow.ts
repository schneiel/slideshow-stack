import type {
	SlideshowRequest,
	SlideshowResponse,
	SlideshowSummary,
	UpdateSlideshowRequest
} from '$lib/api/types';
import { API_ENDPOINTS } from '$lib/api/config';

class ApiResponse<T> {
	success: boolean;
	data: T;
	message?: string;

	constructor(success: boolean, data: T, message?: string) {
		this.success = success;
		this.data = data;
		if (message !== undefined) {
			this.message = message;
		}
	}
}

async function apiRequest<T>(url: string, options: RequestInit = {}): Promise<T> {
	const response = await fetch(url, {
		headers: {
			'Content-Type': 'application/json',
			...options.headers
		},
		...options
	});

	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(`HTTP ${response.status}: ${errorText}`);
	}

	const result: ApiResponse<T> = await response.json();

	if (!result.success) {
		throw new Error(result.message || 'API request failed');
	}

	return result.data;
}

const BASE_URL = API_ENDPOINTS.STORE.SLIDESHOWS;

export const slideshowApi = {
	async getSlideshows(): Promise<SlideshowSummary[]> {
		return apiRequest<SlideshowSummary[]>(BASE_URL);
	},

	async getSlideshow(id: string | undefined): Promise<SlideshowResponse> {
		if (!id) {
			throw new Error('Slideshow ID is required');
		}
		return apiRequest<SlideshowResponse>(`${BASE_URL}/${encodeURIComponent(id)}`);
	},

	async createSlideshow(data: SlideshowRequest): Promise<SlideshowResponse> {
		return apiRequest<SlideshowResponse>(BASE_URL, {
			method: 'POST',
			body: JSON.stringify(data)
		});
	},

	async updateSlideshow(id: string, data: UpdateSlideshowRequest): Promise<SlideshowResponse> {
		return apiRequest<SlideshowResponse>(`${BASE_URL}/${encodeURIComponent(id)}`, {
			method: 'PUT',
			body: JSON.stringify(data)
		});
	},

	async deleteSlideshow(id: string): Promise<{ message: string }> {
		return apiRequest<{ message: string }>(`${BASE_URL}/${encodeURIComponent(id)}`, {
			method: 'DELETE'
		});
	}
};
