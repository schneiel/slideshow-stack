import { showError } from '$lib/stores/toast';

export type ErrorType = 'network' | 'backend-unreachable' | 'validation' | 'unknown';

export interface AppError {
	type: ErrorType;
	message: string;
	details?: string;
	retryable?: boolean;
}

export function handleApiError(error: unknown): AppError {
	if (error instanceof TypeError && error.message.includes('fetch')) {
		return {
			type: 'backend-unreachable',
			message: 'Backend not reachable. Please make sure the API server is running.',
			details: 'Network connection failed - the backend server may not be running or reachable',
			retryable: true
		};
	}

	if (error && typeof error === 'object' && 'message' in error) {
		const message = String(error.message);

		if (message.includes('500')) {
			return {
				type: 'backend-unreachable',
				message: 'Backend server error (HTTP 500). The backend server may not be running.',
				details: message.includes('HTTP 500:')
					? 'The proxy could not connect to the backend server. Please ensure the Store API (port 61532) or Playback Server (port 9247) is running.'
					: message,
				retryable: true
			};
		}

		if (message.includes('502') || message.includes('503') || message.includes('504')) {
			return {
				type: 'backend-unreachable',
				message: 'Backend unavailable. The server is temporarily unable to handle requests.',
				details: message,
				retryable: true
			};
		}

		if (message.includes('404') || message.includes('NOT_FOUND')) {
			return {
				type: 'validation',
				message: 'The requested resource was not found.',
				retryable: false
			};
		}

		if (message.includes('413') || message.includes('PAYLOAD_TOO_LARGE')) {
			return {
				type: 'validation',
				message: 'File too large. Please use smaller files.',
				retryable: false
			};
		}

		if (message.includes('400')) {
			return {
				type: 'validation',
				message: message,
				retryable: false
			};
		}

		if (message.includes('HTTP 500:')) {
			return {
				type: 'backend-unreachable',
				message:
					'Cannot connect to backend server. Please verify the backend services are running.',
				details:
					'The development proxy could not reach the backend. Make sure the Store API is running on port 61532 and the Playback Server is running on port 9247.',
				retryable: true
			};
		}

		return {
			type: 'unknown',
			message: message || 'An unexpected error occurred',
			retryable: true
		};
	}

	return {
		type: 'unknown',
		message: error instanceof Error ? error.message : 'Unknown error occurred',
		retryable: true
	};
}

export function shouldShowErrorState(error: AppError): boolean {
	return error.type === 'backend-unreachable';
}

export function shouldShowToast(error: AppError): boolean {
	return error.type === 'validation' || (error.type === 'network' && !shouldShowErrorState(error));
}

export function showErrorFromApi(error: unknown): AppError {
	const appError = handleApiError(error);

	if (shouldShowToast(appError)) {
		showError(appError.message);
	}

	return appError;
}
