<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { user } from '$lib/auth.js';
	import { toast } from '$lib/toast.js';

	let queries = [];
	let loading = true;

	// Format date for display
	function formatDate(dateString) {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', { 
			year: 'numeric', 
			month: 'short', 
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Truncate text if too long
	function truncateText(text, maxLength = 60) {
		if (!text) return '';
		if (text.length <= maxLength) return text;
		return text.substring(0, maxLength) + '...';
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

	// Handle query click
	function handleQueryClick(queryId) {
		goto(`/dashboard/query/${queryId}`);
	}

	// Load user queries
	onMount(async () => {
		if (!$user) {
			goto('/login');
			return;
		}

		try {
			const response = await fetch('/api/queries', {
				credentials: 'include'
			});
			
			if (response.ok) {
				const data = await response.json();
				queries = data.queries;
			} else {
				toast.error('Failed to load queries');
			}
		} catch (err) {
			toast.error('Failed to load queries');
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Dashboard - GLAD</title>
</svelte:head>

<div class="bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-7xl mx-auto">
		<!-- Header -->
		<div class="mb-8">
			<h1 class="text-3xl font-bold text-gray-800 dark:text-gray-100">Dashboard</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Manage your genetic control queries and view results
			</p>
		</div>

		<!-- Quick Actions -->
		<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-8">
			<h2 class="text-xl font-semibold text-gray-800 dark:text-gray-100 mb-4">Quick Actions</h2>
			<div class="flex flex-wrap gap-4">
				<a
					href="/find"
					class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white font-medium rounded-md transition-colors duration-200"
				>
					<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
					</svg>
					Submit New Query
				</a>
				<a
					href="/explore"
					class="inline-flex items-center px-4 py-2 bg-green-600 hover:bg-green-700 text-white font-medium rounded-md transition-colors duration-200"
				>
					<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
					</svg>
					Explore Cohorts
				</a>
			</div>
		</div>

		<!-- Queries Section -->
		<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg">
			<div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
				<h2 class="text-xl font-semibold text-gray-800 dark:text-gray-100">Your Queries</h2>
			</div>

			{#if loading}
				<div class="p-6">
					<div class="flex justify-center items-center h-32">
						<div class="text-center">
							<svg class="animate-spin h-8 w-8 text-indigo-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							<p class="text-gray-600 dark:text-gray-400">Loading queries...</p>
						</div>
					</div>
				</div>
			{:else if queries.length === 0}
				<div class="p-6 text-center">
					<svg class="w-16 h-16 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
					</svg>
					<p class="text-gray-600 dark:text-gray-400 mb-4">No queries submitted yet</p>
					<a
						href="/find"
						class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white font-medium rounded-md transition-colors duration-200"
					>
						Submit Your First Query
					</a>
				</div>
			{:else}
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-gray-50 dark:bg-gray-700">
							<tr>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
									Title
								</th>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
									Controls
								</th>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
									Status
								</th>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
									Created
								</th>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
									Actions
								</th>
							</tr>
						</thead>
						<tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
							{#each queries as query}
								<tr class="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer" on:click={() => handleQueryClick(query.query_id)}>
									<td class="px-6 py-4">
										<div class="text-sm font-medium text-gray-900 dark:text-white">
											{query.title}
										</div>
										{#if query.description}
											<div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
												{truncateText(query.description, 80)}
											</div>
										{/if}
										{#if query.self_described_latino}
											<div class="text-xs text-indigo-600 dark:text-indigo-400 mt-1">
												Latino-only search
											</div>
										{/if}
									</td>
									<td class="px-6 py-4">
										<div class="text-sm text-gray-900 dark:text-white">
											{query.n_controls.toLocaleString()}
										</div>
									</td>
									<td class="px-6 py-4">
										<span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full {getStatusClass(query.status)}">
											{query.status}
										</span>
									</td>
									<td class="px-6 py-4">
										<div class="text-sm text-gray-900 dark:text-white">
											{formatDate(query.created_at)}
										</div>
									</td>
									<td class="px-6 py-4">
										<button
											on:click|stopPropagation={() => handleQueryClick(query.query_id)}
											class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300 text-sm font-medium cursor-pointer"
										>
											View Details
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	</div>
</div>