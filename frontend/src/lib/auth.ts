import { writable } from 'svelte/store';
import { getCurrentUser, logout as apiLogout } from './api.js';
import { toast } from './toast.js';
import { goto } from '$app/navigation';

interface User {
	username: string;
}

// Auth store to manage user state
export const user = writable<User | null>(null);
export const isLoading = writable(true);

// Check if user is logged in
export async function checkAuth() {
	isLoading.set(true);
	try {
		const currentUser = await getCurrentUser();
		user.set(currentUser);
	} catch (error) {
		user.set(null);
	} finally {
		isLoading.set(false);
	}
}

// Logout function
export async function logout() {
	try {
		await apiLogout();
		user.set(null);
		toast.success('Logged out successfully');
		goto('/');
	} catch (error) {
		toast.error('Failed to logout');
	}
}

// Login success handler
export function setUser(userData: User) {
	user.set(userData);
}

// Initialize auth check when module loads
if (typeof window !== 'undefined') {
	checkAuth();
}