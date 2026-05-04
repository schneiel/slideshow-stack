import type { MediaMetadata, UploadResponse, DeleteMediaResponse } from './types';
import { API_ENDPOINTS } from '$lib/api/config';

class ApiResponse<T> {
	success!: boolean;
	data!: T;
	message?: string;
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

	if (
		options.headers &&
		(options.headers as Record<string, string>)['Content-Type'] === 'application/octet-stream'
	) {
		return response.blob() as unknown as T;
	}

	const result: ApiResponse<T> = await response.json();

	if (!result.success) {
		throw new Error(result.message || 'API request failed');
	}

	return result.data;
}

export class MediaApi {
	private baseUrl = API_ENDPOINTS.STORE.MEDIA;

	async getMedia(): Promise<MediaMetadata[]> {
		return apiRequest<MediaMetadata[]>(this.baseUrl);
	}

	async uploadFiles(files: File[]): Promise<UploadResponse> {
		if (files.length === 0) {
			throw new Error('No files to upload');
		}

		const formData = new FormData();
		files.forEach((file) => {
			formData.append('files', file);
		});

		const response = await fetch(`${this.baseUrl}/upload`, {
			method: 'POST',
			body: formData
		});

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(`HTTP ${response.status}: ${errorText}`);
		}

		const result: ApiResponse<UploadResponse> = await response.json();

		if (!result.success) {
			throw new Error(result.message || 'Upload failed');
		}

		return result.data;
	}

	async deleteMedia(filename: string): Promise<DeleteMediaResponse> {
		return apiRequest<DeleteMediaResponse>(`${this.baseUrl}/${filename}`, {
			method: 'DELETE'
		});
	}

	async downloadMedia(filename: string): Promise<Blob> {
		const response = await fetch(`${this.baseUrl}/${filename}`, {
			headers: {
				'Content-Type': 'application/octet-stream'
			}
		});

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(`HTTP ${response.status}: ${errorText}`);
		}

		return response.blob();
	}
}

export const mediaApi = new MediaApi();
