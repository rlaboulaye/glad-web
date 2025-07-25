<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';
	import PCAVisualization from '$lib/components/PCAVisualization.svelte';
	import IBDVisualization from '$lib/components/IBDVisualization.svelte';
	import MetadataFieldSelector from '$lib/components/MetadataFieldSelector.svelte';
	import GroupSelector from '$lib/components/GroupSelector.svelte';

	// Minimum group size for query submission (privacy protection)
	const MIN_QUERY_GROUP_SIZE = 30;
	// Maximum number of groups to select by default (performance protection)
	const MAX_DEFAULT_SELECTED_GROUPS = 12;

	let data: any[] = [];
	let Plotly: any;
	let loading = true;
	
	// Visualization type selection
	let activeTab = 'pca'; // 'pca' or 'ibd'

	// Metadata fields for selection
	let availableFields = ['phs', 'country', 'region', 'sex', 'ethnicity', 'self_described', 'ibd_community'];
	let selectedFields = new Set(['ethnicity']);
	let asymmetricMode = false;

	// Shared state for groups (with reconciliation)
	let groups: Array<{label: string, size: number}> = [];
	let selectedGroups = new Set<string>();
	let groupsLoading = false;

	// Asymmetric mode state (IBD only) - only Y-axis gets separate state
	// X-axis uses the main selectedFields and selectedGroups
	let yFields = new Set<string>(); 
	let yGroups: Array<{label: string, size: number}> = [];
	let selectedYGroups = new Set<string>();

	// Submit Query section state
	let selectedQueryGroup = '';
	let isSubmitDisabled = true;

	// Component references
	let pcaComponent: PCAVisualization;
	let ibdComponent: IBDVisualization;

	// Submit Query function (placeholder)
	function submitQuery() {
		// TODO: Implement submit query functionality
		console.log('Submit Query clicked for group:', selectedQueryGroup);
	}

	// Event handlers for persistent components
	function handleFieldsChanged(event) {
		selectedFields = event.detail;
		// Trigger PCA component update if needed
		if (pcaComponent) {
			pcaComponent.onFieldsChanged();
		}
	}

	function handleYFieldsChanged(event) {
		yFields = event.detail;
	}

	function handleAsymmetricModeToggled(event) {
		asymmetricMode = event.detail;
		
		if (asymmetricMode) {
			// Copy current symmetric fields to Y fields initially
			yFields = new Set(selectedFields);
		} else {
			// Clear Y fields when returning to symmetric mode
			yFields = new Set();
			selectedYGroups = new Set();
		}
	}

	function handleGroupsChanged(event) {
		selectedGroups = event.detail;
	}

	function handleYGroupsChanged(event) {
		selectedYGroups = event.detail;
	}

	function handleSelectAll() {
		selectedGroups = new Set(groups.map(g => g.label));
	}

	function handleDeselectAll() {
		selectedGroups = new Set();
	}

	// Color function for PCA (from PCAVisualization)
	const COLOR_PALETTE = [
		'#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd',
		'#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf',
		'#aec7e8', '#ffbb78', '#98df8a', '#ff9896', '#c5b0d5',
		'#c49c94', '#f7b6d3', '#c7c7c7', '#dbdb8d', '#9edae5',
		'#393b79', '#5254a3', '#6b6ecf', '#9c9ede', '#637939',
		'#8ca252', '#b5cf6b', '#cedb9c', '#8c6d31', '#bd9e39',
		'#e7ba52', '#e7cb94', '#843c39', '#ad494a', '#d6616b',
		'#e7969c', '#7b4173', '#a55194', '#ce6dbd', '#de9ed6'
	];
	
	const groupColorMap = new Map<string, string>();
	
	function getGroupColor(groupLabel: string): string {
		if (!groupColorMap.has(groupLabel)) {
			const colorIndex = groupColorMap.size % COLOR_PALETTE.length;
			groupColorMap.set(groupLabel, COLOR_PALETTE[colorIndex]);
		}
		return groupColorMap.get(groupLabel)!;
	}

	// Group loading functions
	async function loadPcaGroups() {
		if (!data.length || selectedFields.size === 0) return;
		
		groupsLoading = true;
		try {
			// Generate groups from current data and field selection (from PCA logic)
			const orderedFields = availableFields.filter(f => selectedFields.has(f));
			const grouped = new Map<string, any[]>();

			for (const d of data) {
				const key = orderedFields.length > 0
					? orderedFields.map(f => d[f] ?? 'Unknown').join(' | ')
					: 'All';

				if (!grouped.has(key)) grouped.set(key, []);
				grouped.get(key)!.push(d);
			}

			// Convert to group objects and sort by size (descending)
			const newGroups = Array.from(grouped.entries())
				.map(([label, individuals]) => ({ label, size: individuals.length }))
				.sort((a, b) => b.size - a.size);

			await reconcileGroups(newGroups);
			
		} catch (err) {
			toast.error('Failed to load PCA groups');
			console.error('Error loading PCA groups:', err);
		} finally {
			groupsLoading = false;
		}
	}

	async function loadIbdGroups() {
		if (selectedFields.size === 0) return;
		
		groupsLoading = true;
		try {
			// Generate grouping parameter from selected fields (from IBD logic)
			const orderedFields = availableFields.filter(f => selectedFields.has(f));
			const grouping = orderedFields.join(',');
			
			const res = await fetch(`/api/ibd-groups?grouping=${encodeURIComponent(grouping)}&min_size=30`);
			if (!res.ok) {
				throw new Error('Failed to load IBD groups');
			}
			const data = await res.json();
			
			await reconcileGroups(data.groups);
			
		} catch (err) {
			toast.error('Failed to load IBD groups');
			console.error('Error loading IBD groups:', err);
		} finally {
			groupsLoading = false;
		}
	}

	async function loadAsymmetricYGroups() {
		if (!asymmetricMode || yFields.size === 0) return;
		
		groupsLoading = true;
		try {
			// Load Y-axis groups based on yFields
			const orderedYFields = availableFields.filter(f => yFields.has(f));
			const yGrouping = orderedYFields.join(',');
			
			const res = await fetch(`/api/ibd-groups?grouping=${encodeURIComponent(yGrouping)}&min_size=30`);
			if (!res.ok) {
				throw new Error('Failed to load Y-axis IBD groups');
			}
			const data = await res.json();
			
			// Reconcile Y selections
			const reconciledYSelections = new Set(
				[...selectedYGroups].filter(selection => 
					data.groups.some(group => group.label === selection)
				)
			);
			
			yGroups = data.groups;
			selectedYGroups = reconciledYSelections.size > 0 ? reconciledYSelections : new Set(data.groups.slice(0, 8).map(g => g.label));
			
		} catch (err) {
			toast.error('Failed to load Y-axis IBD groups');
			console.error('Error loading Y-axis IBD groups:', err);
		} finally {
			groupsLoading = false;
		}
	}

	// Smart reconciliation: preserve selections that exist in new groups
	// Only show toast when switching tabs, not when modifying fields within same tab
	let lastActiveTab = activeTab;
	async function reconcileGroups(newGroups: Array<{label: string, size: number}>) {
		const previousSelections = [...selectedGroups];
		const reconciledSelections = new Set(
			previousSelections.filter(selection => 
				newGroups.some(group => group.label === selection)
			)
		);
		
		// Check if groups array was empty before this update
		const wasEmpty = groups.length === 0;
		
		// Update state
		groups = newGroups;
		
		// Only auto-select if groups array was previously empty (initial load or no metadata fields)
		// If user deselected all groups, respect that choice
		if (reconciledSelections.size === 0 && newGroups.length > 0 && wasEmpty) {
			const topGroups = newGroups.slice(0, MAX_DEFAULT_SELECTED_GROUPS).map(g => g.label);
			selectedGroups = new Set(topGroups);
		} else {
			// Apply group limit when switching tabs to prevent expensive computations (especially IBD)
			if (reconciledSelections.size > MAX_DEFAULT_SELECTED_GROUPS) {
				// Keep only the top groups by size when over the limit
				const sortedBySize = Array.from(reconciledSelections)
					.map(label => ({ label, size: newGroups.find(g => g.label === label)?.size || 0 }))
					.sort((a, b) => b.size - a.size)
					.slice(0, MAX_DEFAULT_SELECTED_GROUPS)
					.map(g => g.label);
				selectedGroups = new Set(sortedBySize);
			} else {
				selectedGroups = reconciledSelections;
			}
		}
		
		// Only notify about lost selections if:
		// 1. There were previous selections
		// 2. Some were lost
		// 3. This is happening due to a tab change (not field modification)
		const totalLost = previousSelections.length - selectedGroups.size;
		const unavailableLost = previousSelections.length - reconciledSelections.size;
		const limitLost = reconciledSelections.size - selectedGroups.size;
		const isTabChange = lastActiveTab !== activeTab;
		
		if (totalLost > 0 && previousSelections.length > 0 && isTabChange) {
			if (unavailableLost > 0 && limitLost > 0) {
				toast.info(`${unavailableLost} selections not available in ${activeTab.toUpperCase()}, lost ${limitLost} for performance`);
			} else if (unavailableLost > 0) {
				toast.info(`${unavailableLost} selections not available in ${activeTab.toUpperCase()}`);
			} else if (limitLost > 0) {
				toast.info(`Selections limited to ${MAX_DEFAULT_SELECTED_GROUPS} on switch - lost ${limitLost}`);
			}
		}
		lastActiveTab = activeTab;
	}

	// Reactive: Load groups when fields change or tab switches
	$: if (data.length > 0 && !loading) {
		if (selectedFields.size > 0) {
			if (activeTab === 'pca') {
				loadPcaGroups();
			} else if (activeTab === 'ibd' && !asymmetricMode) {
				loadIbdGroups();
			}
		} else {
			// Clear groups when no metadata fields are selected
			groups = [];
			selectedGroups = new Set();
		}
	}

	// Reactive: Load Y-axis groups when asymmetric mode is enabled and yFields change
	$: if (asymmetricMode && activeTab === 'ibd' && yFields.size > 0) {
		loadAsymmetricYGroups();
	}

	// Reactive: Generate available groups for Query dropdown
	$: availableQueryGroups = (() => {
		if (!groups.length) return [];
		
		// Filter groups by minimum size and format for display
		const MIN_QUERY_GROUP_SIZE = 30;
		const eligibleGroups = asymmetricMode ? 
			// For asymmetric mode, combine X (main) and Y groups
			[...new Set([...selectedGroups, ...selectedYGroups])]
				.map(label => {
					// Find group size from main groups or yGroups
					const mainGroup = groups.find(g => g.label === label);
					const yGroup = yGroups.find(g => g.label === label);
					const size = mainGroup?.size || yGroup?.size || 0;
					return { label, size };
				})
				.filter(group => group.size >= MIN_QUERY_GROUP_SIZE)
				.sort((a, b) => b.size - a.size)
				.map(group => `(${group.size}) ${group.label}`)
			:
			// For symmetric mode, use selected groups
			[...selectedGroups]
				.map(label => {
					const group = groups.find(g => g.label === label);
					return group ? { label: group.label, size: group.size } : null;
				})
				.filter(group => group && group.size >= MIN_QUERY_GROUP_SIZE)
				.sort((a, b) => b.size - a.size)
				.map(group => `(${group.size}) ${group.label}`);
		
		return eligibleGroups;
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
			<!-- Persistent MetadataFieldSelector -->
			<MetadataFieldSelector 
				{availableFields}
				bind:selectedFields
				bind:yFields
				bind:asymmetricMode
				proposeAsymmetric={activeTab === 'ibd'}
				title="Group by Metadata:"
				on:fieldsChanged={handleFieldsChanged}
				on:yFieldsChanged={handleYFieldsChanged}
				on:asymmetricModeToggled={handleAsymmetricModeToggled}
			/>

			<!-- Persistent GroupSelector -->
			<GroupSelector 
				{groups}
				bind:selectedGroups
				loading={groupsLoading}
				description={activeTab === 'pca' ? 'Choose groups to display in the PCA plot' : 'Choose groups to visualize Identity-by-Descent patterns'}
				enableSelectAll={activeTab === 'pca'}
				showColorDots={activeTab === 'pca'}
				getGroupColor={activeTab === 'pca' ? getGroupColor : null}
				{asymmetricMode}
				{yGroups}
				bind:selectedYGroups
				on:groupsChanged={handleGroupsChanged}
				on:yGroupsChanged={handleYGroupsChanged}
				on:selectAll={handleSelectAll}
				on:deselectAll={handleDeselectAll}
			/>

			<!-- Always render both components, but control visibility -->
			<div class:hidden={activeTab !== 'pca'}>
				<PCAVisualization 
					bind:this={pcaComponent}
					{data}
					{availableFields}
					bind:selectedFields
					bind:groups
					bind:selectedGroups
					loading={groupsLoading}
					{Plotly}
					isActive={activeTab === 'pca'}
				/>
			</div>

			<div class:hidden={activeTab !== 'ibd'}>
				<IBDVisualization 
					bind:this={ibdComponent}
					{availableFields}
					bind:selectedFields
					bind:asymmetricMode
					bind:groups
					bind:selectedGroups
					bind:yFields
					bind:yGroups
					bind:selectedYGroups
					loading={groupsLoading}
					{Plotly}
					isActive={activeTab === 'ibd'}
				/>
			</div>

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
