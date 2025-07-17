<script>
	import { resetPassword } from '$lib/api.js';
	import { toast } from '$lib/toast.js';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { PASSWORD_MIN_LENGTH } from '$lib/constants.js';

	let email = '';
	let password = '';
	let confirmPassword = '';
	let loading = false;

	// Check if we have a token in the URL (step 2) or need email (step 1)
	$: token = $page.url.searchParams.get('token');
	$: isStep2 = !!token;

	// Step 1: Request password reset email
	async function handleEmailSubmit() {
		if (loading) return;
		
		if (!email.trim()) {
			toast.error('Email is required');
			return;
		}
		
		if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
			toast.error('Please enter a valid email');
			return;
		}
		
		loading = true;
		
		try {
			await resetPassword(email);
			toast.success('Check your email for reset instructions');
		} catch (err) {
			toast.error('Failed to send reset email. Please try again.');
		} finally {
			loading = false;
		}
	}

	// Step 2: Reset password with token
	async function handlePasswordReset() {
		if (loading) return;
		
		if (!password.trim() || !confirmPassword.trim()) {
			toast.error('Both password fields are required');
			return;
		}
		
		if (password.length < PASSWORD_MIN_LENGTH) {
			toast.error(`Password must be at least ${PASSWORD_MIN_LENGTH} characters`);
			return;
		}
		
		if (password !== confirmPassword) {
			toast.error('Passwords do not match');
			return;
		}
		
		loading = true;
		
		try {
			// Make request to backend with token and new password
			const response = await fetch('/api/auth/reset-password-confirm', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ token, password, confirm: confirmPassword })
			});
			
			const result = await response.json();
			
			if (response.ok) {
				toast.success('Password successfully reset');
				goto('/login');
			} else {
				toast.error(result.error || 'Failed to reset password');
			}
		} catch (err) {
			toast.error('Failed to reset password. Please try again.');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Reset Password - GLAD</title>
</svelte:head>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="sm:mx-auto sm:w-full sm:max-w-md mt-20">
		<h2 class="text-center text-3xl font-bold tracking-tight text-gray-800 dark:text-gray-100">
			Reset Password
		</h2>

		{#if !isStep2}
			<!-- Step 1: Request reset email -->
			<p class="mt-2 text-center text-sm text-gray-600 dark:text-gray-400">
				Enter your email address and we'll send you a link to reset your password.
			</p>
			
			<form on:submit|preventDefault={handleEmailSubmit} class="p-8 shadow-md rounded-lg mt-8 space-y-6 bg-white dark:bg-gray-800">
				<div>
					<label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Email
					</label>
					<input
						id="email"
						name="email"
						type="email"
						bind:value={email}
						placeholder="your@email.com"
						required
						disabled={loading}
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					/>
				</div>

				<div>
					<button
						type="submit"
						disabled={loading}
						class="w-full flex justify-center rounded-md border border-transparent bg-green-400 hover:bg-green-500 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-2 text-sm font-medium text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-green-300 focus:ring-offset-2 transition-colors"
					>
						{#if loading}
							<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							Sending...
						{:else}
							Send Reset Link
						{/if}
					</button>
				</div>
			</form>
		{:else}
			<!-- Step 2: Set new password -->
			<p class="mt-2 text-center text-sm text-gray-600 dark:text-gray-400">
				Enter your new password below.
			</p>
			
			<form on:submit|preventDefault={handlePasswordReset} class="p-8 shadow-md rounded-lg mt-8 space-y-6 bg-white dark:bg-gray-800">
				<div>
					<label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						New Password
					</label>
					<input
						id="password"
						name="password"
						type="password"
						bind:value={password}
						placeholder="New password (min {PASSWORD_MIN_LENGTH} characters)"
						required
						disabled={loading}
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					/>
				</div>

				<div>
					<label for="confirmPassword" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Confirm Password
					</label>
					<input
						id="confirmPassword"
						name="confirmPassword"
						type="password"
						bind:value={confirmPassword}
						placeholder="Confirm new password"
						required
						disabled={loading}
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					/>
				</div>

				<div>
					<button
						type="submit"
						disabled={loading}
						class="w-full flex justify-center rounded-md border border-transparent bg-green-400 hover:bg-green-500 disabled:opacity-50 disabled:cursor-not-allowed px-4 py-2 text-sm font-medium text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-green-300 focus:ring-offset-2 transition-colors"
					>
						{#if loading}
							<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							Resetting...
						{:else}
							Reset Password
						{/if}
					</button>
				</div>
			</form>
		{/if}

		<div class="mt-6 text-center">
			<a href="/login" class="text-sm text-green-400 dark:text-green-300 hover:underline">
				Back to login
			</a>
		</div>
	</div>
</div>