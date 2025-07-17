import { writable } from 'svelte/store';

export interface Toast {
	id: string;
	type: 'success' | 'error' | 'info';
	message: string;
	duration?: number;
}

export const toasts = writable<Toast[]>([]);

export function addToast(type: Toast['type'], message: string, duration = 4000) {
	const id = Math.random().toString(36).substr(2, 9);
	const toast: Toast = { id, type, message, duration };
	
	toasts.update(t => [...t, toast]);
	
	if (duration > 0) {
		setTimeout(() => {
			removeToast(id);
		}, duration);
	}
}

export function removeToast(id: string) {
	toasts.update(t => t.filter(toast => toast.id !== id));
}

export const toast = {
	success: (message: string, duration?: number) => addToast('success', message, duration),
	error: (message: string, duration?: number) => addToast('error', message, duration),
	info: (message: string, duration?: number) => addToast('info', message, duration)
};