<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { user } from '$lib/auth.js';
	import { toast } from '$lib/toast.js';
	import { notifications, fetchNotifications } from '$lib/notifications.js';

	let query = null;
	let loading = true;

	// Get query ID from URL parameters
	$: queryId = $page.params.id;

	// Auto-mark notifications as read for this query
	async function markQueryNotificationsAsRead() {
		try {
			// Get current notifications
			await fetchNotifications();
			
			// Find unread notifications for this query
			const queryNotifications = $notifications.filter(n => 
				!n.is_read && n.query_id === parseInt(queryId)
			);
			
			if (queryNotifications.length > 0) {
				const notificationIds = queryNotifications.map(n => n.notification_id);
				const response = await fetch('/api/notifications/mark-read', {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json',
					},
					credentials: 'include',
					body: JSON.stringify({ notification_ids: notificationIds })
				});
				
				if (response.ok) {
					// Refresh notifications to update the UI
					await fetchNotifications();
				}
			}
		} catch (error) {
			console.error('Failed to mark query notifications as read:', error);
		}
	}

	// Format date for display
	function formatDate(dateString) {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', { 
			year: 'numeric', 
			month: 'long', 
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Get status styling
	function getStatusClass(status) {
		switch (status.toLowerCase()) {
			case 'completed':
				return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
			case 'processing':
				return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
			case 'pending':
				return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
			case 'errored':
				return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
			default:
				return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
		}
	}

	// Get status icon
	function getStatusIcon(status) {
		switch (status.toLowerCase()) {
			case 'completed':
				return 'M5 13l4 4L19 7';
			case 'processing':
				return 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15';
			case 'pending':
				return 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z';
			case 'errored':
				return 'M6 18L18 6M6 6l12 12';
			default:
				return 'M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
		}
	}

	// Load query details - reactive to queryId changes
	async function loadQuery() {
		if (!$user) {
			goto('/login');
			return;
		}

		loading = true;
		query = null;

		try {
			const response = await fetch(`/api/queries/${queryId}`, {
				credentials: 'include'
			});
			
			if (response.ok) {
				query = await response.json();
				// Auto-mark notifications as read for this query
				await markQueryNotificationsAsRead();
			} else if (response.status === 404) {
				toast.error('Query not found');
				goto('/dashboard');
			} else {
				toast.error('Failed to load query details');
				goto('/dashboard');
			}
		} catch (err) {
			toast.error('Failed to load query details');
			goto('/dashboard');
		} finally {
			loading = false;
		}
	}

	// React to queryId changes (when navigating between different query pages)
	$: if (queryId && $user) {
		loadQuery();
	}

	// Initial load
	onMount(() => {
		if (queryId && $user) {
			loadQuery();
		}
	});
</script>

<svelte:head>
	<title>Query Details - GLAD</title>
</svelte:head>

<div class="bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-4xl mx-auto">
		<!-- Breadcrumb -->
		<nav class="mb-8">
			<ol class="flex items-center space-x-2 text-sm">
				<li>
					<a href="/dashboard" class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300">
						Dashboard
					</a>
				</li>
				<li>
					<svg class="w-4 h-4 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
						<path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
					</svg>
				</li>
				<li class="text-gray-900 dark:text-white">
					Query #{queryId}
				</li>
			</ol>
		</nav>

		{#if loading}
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
				<div class="flex justify-center items-center h-32">
					<div class="text-center">
						<svg class="animate-spin h-8 w-8 text-indigo-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						<p class="text-gray-600 dark:text-gray-400">Loading query details...</p>
					</div>
				</div>
			</div>
		{:else if query}
			<!-- Query Header -->
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
				<div class="flex items-center justify-between">
					<div class="flex items-center space-x-3">
						<div class="flex-shrink-0">
							<svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="{getStatusIcon(query.status)}" />
							</svg>
						</div>
						<div>
							<h1 class="text-2xl font-bold text-gray-900 dark:text-white">
								Query #{query.query_id}
							</h1>
							<p class="text-gray-600 dark:text-gray-400">
								Submitted on {formatDate(query.created_at)}
							</p>
						</div>
					</div>
					<div class="flex items-center space-x-3">
						<span class="inline-flex px-3 py-1 text-sm font-semibold rounded-full {getStatusClass(query.status)}">
							{query.status}
						</span>
					</div>
				</div>
			</div>

			<!-- Query Details -->
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
				<!-- Description -->
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Description</h2>
					<p class="text-gray-700 dark:text-gray-300">
						{query.description || 'No description provided'}
					</p>
				</div>

				<!-- Query Parameters -->
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Parameters</h2>
					<dl class="space-y-3">
						<div>
							<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Controls Requested</dt>
							<dd class="text-sm text-gray-900 dark:text-white">{query.n_controls.toLocaleString()}</dd>
						</div>
						<div>
							<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Self-described Latino-only Search</dt>
							<dd class="text-sm text-gray-900 dark:text-white">
								{query.self_described_latino ? 'Yes' : 'No'}
							</dd>
						</div>
						<div>
							<dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Last Updated</dt>
							<dd class="text-sm text-gray-900 dark:text-white">{formatDate(query.status_updated_at)}</dd>
						</div>
					</dl>
				</div>
			</div>

			<!-- Status Information -->
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Status Information</h2>
				
				{#if query.status === 'pending'}
					<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
						<div class="flex items-center">
							<svg class="w-5 h-5 text-blue-600 dark:text-blue-400 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							<p class="text-blue-800 dark:text-blue-200">
								Your query is pending and will be processed soon.
							</p>
						</div>
					</div>
				{:else if query.status === 'processing'}
					<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
						<div class="flex items-center">
							<svg class="w-5 h-5 text-yellow-600 dark:text-yellow-400 mr-3 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
							</svg>
							<p class="text-yellow-800 dark:text-yellow-200">
								Your query is currently being processed. This may take several minutes.
							</p>
						</div>
					</div>
				{:else if query.status === 'completed'}
					<div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
						<div class="flex items-center">
							<svg class="w-5 h-5 text-green-600 dark:text-green-400 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
							</svg>
							<p class="text-green-800 dark:text-green-200">
								Your query has been completed successfully. Results are available for download.
							</p>
						</div>
					</div>
				{:else if query.status === 'errored'}
					<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
						<div class="flex items-center">
							<svg class="w-5 h-5 text-red-600 dark:text-red-400 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
							</svg>
							<p class="text-red-800 dark:text-red-200">
								An error occurred while processing your query. Please try submitting again or contact support.
							</p>
						</div>
					</div>
				{/if}
			</div>

			<!-- Actions -->
			<div class="mt-6 flex justify-between">
				<a
					href="/dashboard"
					class="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 font-medium rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors duration-200"
				>
					<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
					</svg>
					Back to Dashboard
				</a>

				{#if query.status === 'completed'}
					<button
						class="inline-flex items-center px-4 py-2 bg-green-600 hover:bg-green-700 text-white font-medium rounded-md transition-colors duration-200"
						on:click={() => toast.success('Download functionality coming soon!')}
					>
						<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
						</svg>
						Download Results
					</button>
				{/if}
			</div>
		{/if}
	</div>
</div>