// Simple API utilities for the existing Axum backend
export async function login(username: string, password: string) {
	const response = await fetch('/api/auth/login', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		credentials: 'include',
		body: JSON.stringify({ username, password })
	});
	
	const data = await response.json();
	
	if (!response.ok) {
		// Show server message for user errors, generic for server errors
		if ([400, 401, 404, 409].includes(response.status)) {
			throw new Error(data.error || 'Invalid credentials');
		} else {
			throw new Error('Unable to login. Please try again later.');
		}
	}
	
	return data;
}

export async function signup(username: string, email: string, password: string, bio: string) {
	const response = await fetch('/api/auth/signup', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		credentials: 'include',
		body: JSON.stringify({ username, email, password, bio })
	});
	
	const data = await response.json();
	
	if (!response.ok) {
		// Show server message for user errors, generic for server errors
		if ([400, 401, 404, 409].includes(response.status)) {
			throw new Error(data.error || 'Invalid request');
		} else {
			throw new Error('Unable to create account. Please try again later.');
		}
	}
	
	return data;
}

export async function logout() {
	const response = await fetch('/api/auth/logout', {
		method: 'POST',
		credentials: 'include'
	});
	return response.json();
}

export async function resetPassword(email: string) {
	const response = await fetch('/api/auth/reset-password', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		credentials: 'include',
		body: JSON.stringify({ email })
	});
	return response.json();
}

export async function getCurrentUser() {
	try {
		const response = await fetch('/api/auth/me', {
			credentials: 'include'
		});
		if (response.ok) {
			return response.json();
		}
		return null;
	} catch {
		return null;
	}
}