<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';
	import PCAVisualization from '$lib/components/PCAVisualization.svelte';
	import IBDVisualization from '$lib/components/IBDVisualization.svelte';

	// Minimum group size for query submission (privacy protection)
	const MIN_QUERY_GROUP_SIZE = 30;

	let data: any[] = [];
	let Plotly: any;
	let loading = true;
	
	// Visualization type selection
	let activeTab = 'pca'; // 'pca' or 'ibd'

	// Metadata fields for selection
	let availableFields = ['phs', 'country', 'region', 'sex', 'ethnicity', 'self_described', 'ibd_community'];
	let selectedFields = new Set(['ethnicity']);

	// Submit Query section state
	let selectedQueryGroup = '';
	let isSubmitDisabled = true;

	// Component references
	let pcaComponent: PCAVisualization;
	let ibdComponent: IBDVisualization;
	
	// Reactive triggers from components
	let pcaQueryGroupsTrigger = 0;
	let ibdQueryGroupsTrigger = 0;

	// Submit Query function (placeholder)
	function submitQuery() {
		// TODO: Implement submit query functionality
		console.log('Submit Query clicked for group:', selectedQueryGroup);
	}

	// Handle field changes for PCA component
	function handleFieldChange() {
		if (pcaComponent) {
			pcaComponent.onFieldsChanged();
		}
	}

	// Reactive: Generate available groups for Query dropdown
	// Add dependencies on group selections to trigger updates
	$: availableQueryGroups = (() => {
		if (activeTab === 'pca') {
			// Trigger update when PCA fields or group selections change
			selectedFields; // reactive dependency
			pcaQueryGroupsTrigger; // reactive dependency for PCA group selections
			return pcaComponent ? pcaComponent.getAvailableQueryGroups() : [];
		} else if (activeTab === 'ibd') {
			// Trigger update when IBD group selections change
			ibdQueryGroupsTrigger; // reactive dependency for IBD group selections
			return ibdComponent ? ibdComponent.getAvailableQueryGroups() : [];
		}
		return [];
	})();

	// Reactive: Update submit button state and reset selection when groups change
	$: {
		isSubmitDisabled = !selectedQueryGroup || !availableQueryGroups.includes(selectedQueryGroup);
		// Reset selection if current selection is no longer available
		if (selectedQueryGroup && !availableQueryGroups.includes(selectedQueryGroup)) {
			selectedQueryGroup = '';
		}
	}

	onMount(async () => {
		try {
			const res = await fetch('/api/pca-data');
			if (!res.ok) {
				throw new Error('Failed to load PCA data');
			}
			data = await res.json();
			Plotly = (await import('plotly.js-dist-min')).default;
		} catch (err) {
			toast.error('Failed to load PCA data');
			console.error('Error loading PCA data:', err);
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Explore - GLAD</title>
</svelte:head>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-7xl mx-auto">
		<!-- Header -->
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-gray-800 dark:text-gray-100">Explore Cohorts</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Visualize genetic diversity and relatedness patterns
			</p>
		</div>

		<!-- Visualization Type Tabs -->
		<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-2 mb-6">
			<div class="flex space-x-1" role="tablist">
				<button
					class="flex-1 py-3 px-6 text-sm font-medium rounded-md transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500
						{activeTab === 'pca' 
							? 'bg-indigo-600 text-white shadow-sm' 
							: 'text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'}"
					role="tab"
					aria-selected={activeTab === 'pca'}
					on:click={() => activeTab = 'pca'}
				>
					PCA Analysis
				</button>
				<button
					class="flex-1 py-3 px-6 text-sm font-medium rounded-md transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500
						{activeTab === 'ibd' 
							? 'bg-indigo-600 text-white shadow-sm' 
							: 'text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'}"
					role="tab"
					aria-selected={activeTab === 'ibd'}
					on:click={() => activeTab = 'ibd'}
				>
					IBD Heatmap
				</button>
			</div>
		</div>

		{#if loading}
			<div class="flex justify-center items-center h-64">
				<div class="text-center">
					<svg class="animate-spin h-8 w-8 text-indigo-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					<p class="text-gray-600 dark:text-gray-400">Loading data...</p>
				</div>
			</div>
		{:else}
			<!-- PCA Analysis Tab -->
			{#if activeTab === 'pca'}
				<PCAVisualization 
					bind:this={pcaComponent}
					{data}
					{availableFields}
					bind:selectedFields
					{Plotly}
					isActive={activeTab === 'pca'}
					bind:queryGroupsUpdateTrigger={pcaQueryGroupsTrigger}
				/>
			{/if}

			<!-- IBD Heatmap Tab -->
			{#if activeTab === 'ibd'}
				<IBDVisualization 
					bind:this={ibdComponent}
					{availableFields}
					{Plotly}
					isActive={activeTab === 'ibd'}
					bind:queryGroupsUpdateTrigger={ibdQueryGroupsTrigger}
				/>
			{/if}

			<!-- Submit Query for Group Summary Data -->
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
				<div class="space-y-4">
					<h3 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Submit Query for Group Summary Data</h3>
					
					<div class="flex items-end space-x-4">
						<div class="flex-1">
							<label for="group-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
								Select Group:
							</label>
							<select 
								id="group-select"
								bind:value={selectedQueryGroup}
								class="w-full p-3 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
							>
								<option value="">Choose a group...</option>
								{#each availableQueryGroups as group}
									<option value={group}>{group}</option>
								{/each}
							</select>
						</div>
						
						<div class="flex-shrink-0">
							<button
								class="px-6 py-3 text-sm font-medium rounded-md transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500
									{isSubmitDisabled 
										? 'bg-gray-300 dark:bg-gray-600 text-gray-500 dark:text-gray-400 cursor-not-allowed' 
										: 'bg-indigo-600 text-white hover:bg-indigo-700'}"
								disabled={isSubmitDisabled}
								on:click={submitQuery}
							>
								Submit
							</button>
						</div>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>