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

	// File upload variables
	let selectedFile = null;
	let fileUploadLoading = false;
	let dragOver = false;

	// File validation
	const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB
	
	function validateFile(file) {
		if (file.size > MAX_FILE_SIZE) {
			return 'File size must be less than 10MB';
		}
		// For now, auto-pass all files - validation can be extended later
		return null;
	}

	// File upload handlers
	function handleFileSelect(event) {
		const file = event.target.files[0];
		if (file) {
			const error = validateFile(file);
			if (error) {
				toast.error(error);
				return;
			}
			selectedFile = file;
		}
	}

	function handleDragOver(event) {
		event.preventDefault();
		dragOver = true;
	}

	function handleDragLeave(event) {
		event.preventDefault();
		dragOver = false;
	}

	function handleDrop(event) {
		event.preventDefault();
		dragOver = false;
		
		const files = event.dataTransfer.files;
		if (files.length > 0) {
			const file = files[0];
			const error = validateFile(file);
			if (error) {
				toast.error(error);
				return;
			}
			selectedFile = file;
		}
	}

	function removeFile() {
		selectedFile = null;
	}

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

		if (!selectedFile) {
			toast.error('Query samples embedding file is required');
			return;
		}

		loading = true;

		try {
			// Create FormData to handle file upload
			const formData = new FormData();
			formData.append('description', description.trim());
			formData.append('self_described_latino', selfDescribedLatino.toString());
			formData.append('n_controls', nControls.toString());
			formData.append('excluded_cohorts', JSON.stringify(selectedCohorts));
			formData.append('query_file', selectedFile);

			const response = await fetch('/api/find-controls', {
				method: 'POST',
				credentials: 'include',
				body: formData // No Content-Type header needed for FormData
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

<div class="bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
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

					<!-- File Upload Section -->
					<div>
						<h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-4">
							Query Samples Embedding:
						</h3>
						
						<div 
							class="border-2 border-dashed rounded-lg p-6 text-center transition-colors duration-200
								{dragOver 
									? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20' 
									: 'border-gray-300 dark:border-gray-600 hover:border-indigo-400 dark:hover:border-indigo-500'}"
							on:dragover={handleDragOver}
							on:dragleave={handleDragLeave}
							on:drop={handleDrop}
							role="button"
							tabindex="0"
						>
							{#if selectedFile}
								<!-- File selected state -->
								<div class="flex items-center justify-between p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-700 rounded-md">
									<div class="flex items-center space-x-3">
										<svg class="h-8 w-8 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
										</svg>
										<div class="text-left">
											<p class="text-sm font-medium text-gray-900 dark:text-gray-100">{selectedFile.name}</p>
											<p class="text-xs text-gray-500 dark:text-gray-400">
												{(selectedFile.size / 1024 / 1024).toFixed(2)} MB
											</p>
										</div>
									</div>
									<button
										type="button"
										on:click={removeFile}
										class="text-red-500 hover:text-red-700 dark:hover:text-red-300"
									>
										<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
										</svg>
									</button>
								</div>
							{:else}
								<!-- Upload area -->
								<svg class="mx-auto h-12 w-12 text-gray-400 dark:text-gray-500 mb-4" stroke="currentColor" fill="none" viewBox="0 0 48 48">
									<path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
								</svg>
								<div class="text-gray-600 dark:text-gray-400">
									<label for="file-upload" class="cursor-pointer">
										<span class="text-indigo-600 dark:text-indigo-400 font-medium hover:text-indigo-500">
											Click to upload
										</span>
										<span> or drag and drop</span>
										<input
											id="file-upload"
											type="file"
											class="sr-only"
											on:change={handleFileSelect}
											disabled={loading}
										/>
									</label>
								</div>
								<p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
									Maximum file size: 10MB
								</p>
							{/if}
						</div>
						
						<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
							Upload your query samples embedding file to find matching controls.
						</p>
					</div>

					<!-- Submit button -->
					<div class="flex justify-end">
						<button
							type="submit"
							disabled={loading || !$user || !selectedFile}
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