<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { user } from '$lib/auth.js';
	import { toast } from '$lib/toast.js';

	let description = '';
	let selfDescribedLatino = false;
	let nControls = 100;
	let cohorts = [];
	let selectedCohorts = [];
	let loading = false;
	let cohortsLoading = true;

	// Load cohorts on mount
	onMount(async () => {
		try {
			const response = await fetch('/api/cohorts', {
				credentials: 'include'
			});
			
			if (response.ok) {
				const data = await response.json();
				cohorts = data.cohorts;
			} else {
				toast.error('Failed to load cohorts');
			}
		} catch (err) {
			toast.error('Failed to load cohorts');
		} finally {
			cohortsLoading = false;
		}
	});

	// Handle cohort selection
	function handleCohortChange(cohortName, checked) {
		if (checked) {
			selectedCohorts = [...selectedCohorts, cohortName];
		} else {
			selectedCohorts = selectedCohorts.filter(name => name !== cohortName);
		}
	}

	// Handle form submission
	async function handleSubmit() {
		if (loading) return;

		// Validate form
		if (!description.trim()) {
			toast.error('Description is required');
			return;
		}

		if (description.trim().length < 4) {
			toast.error('Description must be at least 4 characters long');
			return;
		}

		if (nControls <= 0) {
			toast.error('Number of controls must be greater than 0');
			return;
		}

		loading = true;

		try {
			const response = await fetch('/api/find-controls', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({
					description: description.trim(),
					self_described_latino: selfDescribedLatino,
					n_controls: nControls,
					excluded_cohorts: selectedCohorts
				})
			});

			const result = await response.json();

			if (response.ok) {
				toast.success('Query submitted successfully');
				// Redirect to query page (we'll need to implement this later)
				// goto(`/query/${result.query_id}`);
				goto('/dashboard'); // For now, redirect to dashboard
			} else {
				toast.error(result.error || 'Failed to submit query');
			}
		} catch (err) {
			toast.error('Failed to submit query. Please try again.');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Find Controls - GLAD</title>
</svelte:head>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-4xl mx-auto">
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-gray-800 dark:text-gray-100">Find Controls</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Submit a query to find genetic controls for your research
			</p>
		</div>

		<!-- Status Messages -->
		{#if !$user}
			<div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
				<strong>Error:</strong> You must be logged in to submit queries.
			</div>
		{:else}
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-8">
				<form on:submit|preventDefault={handleSubmit} class="space-y-6">
					<!-- Description -->
					<div>
						<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							Project Description
						</label>
						<input
							id="description"
							type="text"
							bind:value={description}
							disabled={loading}
							placeholder="Give a brief description of your project and need for additional controls."
							class="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm placeholder-gray-400 focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
						/>
					</div>

					<!-- Self-described Latino checkbox -->
					<div class="flex items-center">
						<input
							id="selfDescribedLatino"
							type="checkbox"
							bind:checked={selfDescribedLatino}
							disabled={loading}
							class="rounded border-gray-300 text-indigo-600 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50"
						/>
						<label for="selfDescribedLatino" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
							Restrict search to self-described latinos only
						</label>
					</div>

					<!-- Number of controls -->
					<div>
						<label for="nControls" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							How many controls would you like to find?
						</label>
						<input
							id="nControls"
							type="number"
							bind:value={nControls}
							disabled={loading}
							min="1"
							max="10000"
							class="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 px-3 py-2 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm text-gray-900 dark:text-white disabled:opacity-50"
						/>
					</div>

					<!-- Cohorts selection -->
					<div>
						<h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-4">
							Select cohorts to exclude from matching procedure:
						</h3>
						
						{#if cohortsLoading}
							<p class="text-gray-600 dark:text-gray-400">Loading cohorts...</p>
						{:else if cohorts.length === 0}
							<p class="text-red-600 dark:text-red-400">Failed to load cohorts. Please try again.</p>
						{:else}
							<div class="max-h-64 overflow-y-auto border border-gray-300 dark:border-gray-600 rounded-md p-4 bg-gray-50 dark:bg-gray-700">
								<div class="space-y-2">
									{#each cohorts as cohort}
										<div class="flex items-center">
											<input
												id="cohort-{cohort.cohort_id}"
												type="checkbox"
												disabled={loading}
												on:change={(e) => handleCohortChange(cohort.cohort_name, e.target.checked)}
												class="rounded border-gray-300 text-indigo-600 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50"
											/>
											<label for="cohort-{cohort.cohort_id}" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
												{cohort.cohort_name}
											</label>
										</div>
									{/each}
								</div>
							</div>
							
							{#if selectedCohorts.length > 0}
								<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
									Selected cohorts to exclude: {selectedCohorts.join(', ')}
								</p>
							{/if}
						{/if}
					</div>

					<!-- Submit button -->
					<div class="flex justify-end">
						<button
							type="submit"
							disabled={loading || !$user}
							class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
						>
							{#if loading}
								<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								Submitting...
							{:else}
								Submit Query
							{/if}
						</button>
					</div>
				</form>
			</div>
		{/if}
	</div>
</div>