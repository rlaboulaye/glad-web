<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { user } from '$lib/auth.js';
	import { toast } from '$lib/toast.js';
	import { PASSWORD_MIN_LENGTH } from '$lib/constants.js';

	let currentUser = null;
	let loading = false;
	let initialLoading = true;
	
	// Form fields
	let bio = '';
	let email = '';
	let password = '';
	let confirmPassword = '';
	let emailNotifications = false;

	// Load current user data
	onMount(async () => {
		try {
			const response = await fetch('/api/auth/me', {
				credentials: 'include'
			});
			
			if (response.ok) {
				currentUser = await response.json();
				bio = currentUser.bio || '';
				email = currentUser.email || '';
				emailNotifications = currentUser.email_notifications || false;
			} else {
				toast.error('Failed to load user data');
				goto('/login');
			}
		} catch (err) {
			toast.error('Failed to load user data');
			goto('/login');
		} finally {
			initialLoading = false;
		}
	});

	async function handleUpdateSettings() {
		if (loading) return;

		// Validate password if provided
		if (password) {
			if (password.length < PASSWORD_MIN_LENGTH) {
				toast.error(`Password must be at least ${PASSWORD_MIN_LENGTH} characters`);
				return;
			}
			
			if (password !== confirmPassword) {
				toast.error('Passwords do not match');
				return;
			}
		}

		loading = true;

		try {
			const response = await fetch('/api/auth/settings', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({
					bio: bio.trim(),
					email: email.trim(),
					password: password,
					confirm_password: confirmPassword,
					email_notifications: emailNotifications
				})
			});

			const result = await response.json();

			if (response.ok) {
				toast.success('Settings updated successfully');
				// Clear password fields after successful update
				password = '';
				confirmPassword = '';
				// Update current user data
				currentUser = { ...currentUser, bio: bio.trim(), email: email.trim(), email_notifications: emailNotifications };
			} else {
				toast.error(result.error || 'Failed to update settings');
			}
		} catch (err) {
			toast.error('Failed to update settings. Please try again.');
		} finally {
			loading = false;
		}
	}

	async function handleLogout() {
		try {
			await fetch('/api/auth/logout', {
				method: 'POST',
				credentials: 'include'
			});
			
			// Clear user state
			user.set(null);
			goto('/');
		} catch (err) {
			toast.error('Failed to logout');
		}
	}
</script>

<svelte:head>
	<title>Settings - GLAD</title>
</svelte:head>

<div class="bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full mx-auto mt-10">
		<h1 class="text-center text-3xl font-bold text-gray-800 dark:text-gray-100 mb-6">
			Profile Settings
		</h1>

		{#if initialLoading}
			<p class="text-center text-gray-600 dark:text-gray-400">Loading user settings...</p>
		{:else if currentUser}
			<form on:submit|preventDefault={handleUpdateSettings} class="p-8 shadow-md rounded-lg mt-8 space-y-6 bg-white dark:bg-gray-800">
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Username
					</label>
					<input
						disabled
						type="text"
						placeholder="Your Username"
						value={currentUser.username}
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-gray-100 dark:bg-gray-600 px-3 py-2 shadow-sm placeholder-gray-400 text-gray-500 dark:text-gray-400 sm:text-sm cursor-not-allowed"
					/>
				</div>

				<div>
					<label for="bio" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Bio
					</label>
					<textarea
						id="bio"
						name="bio"
						rows="5"
						bind:value={bio}
						disabled={loading}
						placeholder="Short bio about you and your work"
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					></textarea>
				</div>

				<div>
					<label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Email
					</label>
					<input
						id="email"
						name="email"
						type="email"
						bind:value={email}
						disabled={loading}
						placeholder="Email"
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					/>
				</div>

				<div>
					<div class="flex items-center justify-between">
						<div>
							<label for="emailNotifications" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
								Email Notifications
							</label>
							<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
								Receive email notifications when your queries are completed or errored
							</p>
						</div>
						<div class="ml-4">
							<button
								type="button"
								on:click={() => emailNotifications = !emailNotifications}
								disabled={loading}
								class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed {emailNotifications ? 'bg-indigo-600' : 'bg-gray-200 dark:bg-gray-600'}"
								role="switch"
								aria-checked={emailNotifications}
								aria-labelledby="emailNotifications"
							>
								<span 
									class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {emailNotifications ? 'translate-x-5' : 'translate-x-0'}"
								></span>
							</button>
						</div>
					</div>
				</div>

				<div>
					<label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Password
					</label>
					<input
						id="password"
						name="password"
						type="password"
						bind:value={password}
						disabled={loading}
						placeholder="New Password (leave blank to keep current)"
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					/>
					<input
						id="confirmPassword"
						name="confirmPassword"
						type="password"
						bind:value={confirmPassword}
						disabled={loading}
						placeholder="Confirm Password"
						class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
					/>
				</div>

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
						Updating...
					{:else}
						Update Settings
					{/if}
				</button>
			</form>
		{:else}
			<p class="text-center text-red-500">
				There was a problem while fetching settings, try again later
			</p>
		{/if}

		<hr class="my-8 border-gray-300 dark:border-gray-700"/>

		<button 
			type="button" 
			on:click={handleLogout}
			class="w-full bg-red-500 hover:bg-red-600 text-white py-2 px-4 rounded-lg transition-colors"
		>
			Logout
		</button>
	</div>
</div>