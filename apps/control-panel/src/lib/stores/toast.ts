import { writable } from 'svelte/store';

export interface Toast {
	id: string;
	type: 'success' | 'error' | 'warning' | 'info';
	message: string;
	duration?: number;
	dismissible?: boolean;
}

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	return {
		subscribe,
		add: (toast: Omit<Toast, 'id'>) => {
			const id = Math.random().toString(36).substring(2, 9);
			const newToast: Toast = {
				id,
				...toast
			};

			update((toasts: Toast[]) => [...toasts, newToast]);

			if (newToast.duration !== 0) {
				setTimeout(() => {
					update((toasts: Toast[]) => toasts.filter((t: Toast) => t.id !== id));
				}, newToast.duration || 4000);
			}

			return id;
		},
		remove: (id: string) => {
			update((toasts: Toast[]) => toasts.filter((t: Toast) => t.id !== id));
		},
		clear: () => {
			update(() => []);
		}
	};
}

export const toast = createToastStore();

export const showSuccess = (message: string, duration?: number) =>
	toast.add({
		type: 'success',
		message,
		...(duration !== undefined ? { duration } : {})
	});

export const showError = (message: string, duration?: number) =>
	toast.add({
		type: 'error',
		message,
		...(duration !== undefined ? { duration } : {})
	});

export const showWarning = (message: string, duration?: number) =>
	toast.add({
		type: 'warning',
		message,
		...(duration !== undefined ? { duration } : {})
	});

export const showInfo = (message: string, duration?: number) =>
	toast.add({
		type: 'info',
		message,
		...(duration !== undefined ? { duration } : {})
	});
