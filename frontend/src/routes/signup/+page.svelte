<script>
	import { signup } from '$lib/api.js';
	import { toast } from '$lib/toast.js';
	import { goto } from '$app/navigation';
	import { USERNAME_MIN_LENGTH, PASSWORD_MIN_LENGTH } from '$lib/constants.js';

	let username = '';
	let email = '';
	let password = '';
	let bio = '';
	let loading = false;

	// Client-side validation
	function validateForm() {
		if (!username.trim()) return 'Username is required';
		if (username.length < USERNAME_MIN_LENGTH) return `Username must be at least ${USERNAME_MIN_LENGTH} characters`;
		if (!email.trim()) return 'Email is required';
		if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) return 'Please enter a valid email';
		if (!password.trim()) return 'Password is required';
		if (password.length < PASSWORD_MIN_LENGTH) return `Password must be at least ${PASSWORD_MIN_LENGTH} characters`;
		return null;
	}

	async function handleSubmit() {
		if (loading) return;
		
		const validationError = validateForm();
		if (validationError) {
			toast.error(validationError);
			return;
		}
		
		loading = true;
		
		try {
			const result = await signup(username, email, password, bio);
			toast.success('Account created successfully!');
			goto('/login');
		} catch (err) {
			toast.error(err.message || 'Failed to create account. Please try again.');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Sign up - GLAD</title>
</svelte:head>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="sm:mx-auto sm:w-full sm:max-w-md mt-20">
		<h2 class="text-center text-3xl font-bold tracking-tight text-gray-800 dark:text-gray-100">
			Sign up
		</h2>

		<p class="mt-2 text-center text-sm text-gray-600 dark:text-gray-400">
			<a
				href="/login"
				class="text-green-400 dark:text-green-300 hover:underline"
			>
				Have an account?
			</a>
		</p>

		<form on:submit|preventDefault={handleSubmit} class="p-8 shadow-md rounded-lg mt-8 space-y-6 bg-white dark:bg-gray-800">
			<div>
				<label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Username *
				</label>
				<input
					id="username"
					name="username"
					type="text"
					bind:value={username}
					placeholder="Your Username"
					required
					disabled={loading}
					class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
				/>
			</div>

			<div>
				<label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Email *
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
				<label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Password *
				</label>
				<input
					id="password"
					name="password"
					type="password"
					bind:value={password}
					placeholder="Password (min {PASSWORD_MIN_LENGTH} characters)"
					required
					disabled={loading}
					class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
				/>
			</div>

			<div>
				<label for="bio" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Bio (optional)
				</label>
				<textarea
					id="bio"
					name="bio"
					rows="4"
					bind:value={bio}
					placeholder="Short bio about you and your work"
					disabled={loading}
					class="mt-1 mb-4 block w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
				></textarea>
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
						Creating account...
					{:else}
						Sign up
					{/if}
				</button>
			</div>
		</form>
	</div>
</div>