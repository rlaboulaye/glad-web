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
	let availableFields = ['phs', 'country', 'region', 'sex', 'ethnicity', 'ethnicity_source', 'ibd_community'];
	let selectedFields = new Set(['phs']);
	let crossGroupingMode = false;

	// Shared state for groups (with reconciliation)
	let groups: Array<{label: string, size: number}> = [];
	let selectedGroups = new Set<string>();
	let groupsLoading = false;
	
	// State tracking for group reconciliation
	let groupsReconciledForFields = new Set<string>(); // Track which fields the current groups represent
	let secondaryGroupsReconciledForFields = new Set<string>(); // Track which fields the current secondary groups represent

	// Cross-Grouping mode state
	// Primary axis uses the main selectedFields and selectedGroups
	let secondaryFields = new Set<string>(); 
	let secondaryGroups: Array<{label: string, size: number}> = [];
	let selectedSecondaryGroups = new Set<string>();

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

	function handleSecondaryFieldsChanged(event) {
		secondaryFields = event.detail;
	}

	function handleCrossGroupingModeToggled(event) {
		crossGroupingMode = event.detail;
		
		if (crossGroupingMode) {
			// Copy current primary fields to secondary fields initially
			secondaryFields = new Set(selectedFields);
		} else {
			// Clear secondary fields when returning to primary mode
			secondaryFields = new Set();
			selectedSecondaryGroups = new Set();
		}
	}

	function handleGroupsChanged(event) {
		selectedGroups = event.detail;
	}

	function handleSecondaryGroupsChanged(event) {
		selectedSecondaryGroups = event.detail;
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

	// Helper function to generate PCA groups from data and fields
	function generatePcaGroups(fields: Set<string>): Array<{label: string, size: number}> {
		if (!data.length || fields.size === 0) return [];
		
		// Generate groups from current data and field selection
		const orderedFields = availableFields.filter(f => fields.has(f));
		const grouped = new Map<string, any[]>();

		for (const d of data) {
			const key = orderedFields.length > 0
				? orderedFields.map(f => d[f] ?? 'Unknown').join(' | ')
				: 'All';

			if (!grouped.has(key)) grouped.set(key, []);
			grouped.get(key)!.push(d);
		}

		// Convert to group objects and sort by size (descending)
		return Array.from(grouped.entries())
			.map(([label, individuals]) => ({ label, size: individuals.length }))
			.sort((a, b) => b.size - a.size);
	}

	// Group loading functions
	async function loadPcaGroups() {
		if (!data.length || selectedFields.size === 0) return;
		
		groupsLoading = true;
		try {
			const newGroups = generatePcaGroups(selectedFields);
			await reconcileGroups(newGroups);
			
		} catch (err) {
			toast.error('Failed to load PCA groups');
			console.error('Error loading PCA groups:', err);
		} finally {
			groupsLoading = false;
		}
	}

	async function loadPcaSecondaryGroups() {
		if (!data.length || secondaryFields.size === 0) return;
		
		groupsLoading = true;
		try {
			const newSecondaryGroups = generatePcaGroups(secondaryFields);
			const previousSecondarySelections = [...selectedSecondaryGroups];
			const wasSecondaryEmpty = secondaryGroups.length === 0;
			
			// Use the generic reconciliation function
			const reconciledSecondarySelections = reconcileGroupSelections(
				previousSecondarySelections,
				newSecondaryGroups,
				lastSecondaryFields,
				secondaryFields,
				wasSecondaryEmpty,
				8 // Use 8 for secondary groups default
			);
			
			secondaryGroups = newSecondaryGroups;
			selectedSecondaryGroups = reconciledSecondarySelections;
			
			// Update tracking variables
			lastSecondaryFields = new Set(secondaryFields);
			
			// Mark secondary groups as reconciled for current field selection
			secondaryGroupsReconciledForFields = new Set(secondaryFields);
			
		} catch (err) {
			toast.error('Failed to load secondary-axis PCA groups');
			console.error('Error loading secondary-axis PCA groups:', err);
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

	async function loadSecondaryGroups() {
		if (!crossGroupingMode || secondaryFields.size === 0) return;
		
		groupsLoading = true;
		try {
			// Load secondary-axis groups based on secondaryFields
			const orderedSecondaryFields = availableFields.filter(f => secondaryFields.has(f));
			const secondaryGrouping = orderedSecondaryFields.join(',');
			
			const res = await fetch(`/api/ibd-groups?grouping=${encodeURIComponent(secondaryGrouping)}&min_size=30`);
			if (!res.ok) {
				throw new Error('Failed to load secondary-axis IBD groups');
			}
			const data = await res.json();
			
			const previousSecondarySelections = [...selectedSecondaryGroups];
			const wasSecondaryEmpty = secondaryGroups.length === 0;
			
			// Use the generic reconciliation function
			const reconciledSecondarySelections = reconcileGroupSelections(
				previousSecondarySelections,
				data.groups,
				lastSecondaryFields,
				secondaryFields,
				wasSecondaryEmpty,
				8 // Use 8 for secondary groups default
			);
			
			secondaryGroups = data.groups;
			selectedSecondaryGroups = reconciledSecondarySelections;
			
			// Update tracking variables
			lastSecondaryFields = new Set(secondaryFields);
			
			// Mark secondary groups as reconciled for current field selection
			secondaryGroupsReconciledForFields = new Set(secondaryFields);
			
		} catch (err) {
			toast.error('Failed to load secondary-axis IBD groups');
			console.error('Error loading secondary-axis IBD groups:', err);
		} finally {
			groupsLoading = false;
		}
	}

	// Helper function to compare sets for equality
	function setsEqual<T>(a: Set<T>, b: Set<T>): boolean {
		return a.size === b.size && [...a].every(x => b.has(x));
	}

	// Enhanced group reconciliation: find semantic parent-child relationships
	// Parse group labels into field-value maps using canonical field order
	function parseGroupLabel(label: string, currentFields: Set<string>): Record<string, string> {
		const values = label.split(' | ');
		const fieldMap: Record<string, string> = {};
		
		// Use canonical field order to match values to fields
		let fieldIndex = 0;
		for (const field of availableFields) {
			if (fieldIndex < values.length && currentFields.has(field)) {
				fieldMap[field] = values[fieldIndex];
				fieldIndex++;
			}
		}
		return fieldMap;
	}

	// Check if newGroup is a semantic "child" of oldGroup (contains all old field values)
	function isChildGroup(oldGroupMap: Record<string, string>, newGroupMap: Record<string, string>): boolean {
		return Object.entries(oldGroupMap).every(([field, value]) => 
			newGroupMap[field] === value
		);
	}

	// Find child groups for enhanced reconciliation (adding fields)
	function findChildGroups(oldSelections: string[], newGroups: Array<{label: string, size: number}>, oldFields: Set<string>, newFields: Set<string>): Set<string> {
		const childSelections = new Set<string>();
		
		for (const oldSelection of oldSelections) {
			const oldGroupMap = parseGroupLabel(oldSelection, oldFields);
			
			// Find all new groups that are children of this old selection
			for (const newGroup of newGroups) {
				const newGroupMap = parseGroupLabel(newGroup.label, newFields);
				
				if (isChildGroup(oldGroupMap, newGroupMap)) {
					childSelections.add(newGroup.label);
				}
			}
		}
		
		return childSelections;
	}

	// Find parent groups for "round up" reconciliation (removing fields)
	function findParentGroups(oldSelections: string[], newGroups: Array<{label: string, size: number}>, oldFields: Set<string>, newFields: Set<string>): Set<string> {
		const parentSelections = new Set<string>();
		
		// Group old selections by their potential parent groups
		const parentGroupMap = new Map<string, string[]>();
		
		for (const oldSelection of oldSelections) {
			const oldGroupMap = parseGroupLabel(oldSelection, oldFields);
			
			// Create parent key by only using fields that exist in newFields
			const parentKey = Object.entries(oldGroupMap)
				.filter(([field, _]) => newFields.has(field))
				.sort(([a], [b]) => availableFields.indexOf(a) - availableFields.indexOf(b)) // Use canonical order
				.map(([_, value]) => value)
				.join(' | ');
			
			if (parentKey) {
				if (!parentGroupMap.has(parentKey)) {
					parentGroupMap.set(parentKey, []);
				}
				parentGroupMap.get(parentKey)!.push(oldSelection);
			}
		}
		
		// For each potential parent, check if it exists in newGroups and select it
		for (const [parentKey, childSelections] of parentGroupMap.entries()) {
			// Find the actual parent group in newGroups
			const parentGroup = newGroups.find(group => group.label === parentKey);
			if (parentGroup) {
				parentSelections.add(parentGroup.label);
			}
		}
		
		return parentSelections;
	}

	// Generic reconciliation function - can be used for both primary and secondary groups
	function reconcileGroupSelections(
		previousSelections: string[], 
		newGroups: Array<{label: string, size: number}>,
		previousFields: Set<string>,
		currentFields: Set<string>,
		wasGroupsEmpty: boolean,
		maxDefaultSelected: number = MAX_DEFAULT_SELECTED_GROUPS
	): Set<string> {
		// Try exact matches first
		const exactMatches = new Set(
			previousSelections.filter(selection => 
				newGroups.some(group => group.label === selection)
			)
		);

		// Try semantic matching if exact matches failed and fields changed
		let reconciledSelections = exactMatches;
		if (exactMatches.size === 0 && previousSelections.length > 0 && !setsEqual(previousFields, currentFields)) {
			// Determine if fields were added or removed
			const oldFieldCount = previousFields.size;
			const newFieldCount = currentFields.size;
			
			if (newFieldCount > oldFieldCount) {
				// Fields were added - find child groups (more specific)
				reconciledSelections = findChildGroups(previousSelections, newGroups, previousFields, currentFields);
			} else if (newFieldCount < oldFieldCount) {
				// Fields were removed - find parent groups (less specific, "round up")
				reconciledSelections = findParentGroups(previousSelections, newGroups, previousFields, currentFields);
			} else {
				// Same number of fields but different fields - try both approaches
				const childMatches = findChildGroups(previousSelections, newGroups, previousFields, currentFields);
				const parentMatches = findParentGroups(previousSelections, newGroups, previousFields, currentFields);
				reconciledSelections = childMatches.size > 0 ? childMatches : parentMatches;
			}
		}
		
		// Only auto-select if groups array was previously empty (initial load or no metadata fields)
		// If user deselected all groups, respect that choice
		if (reconciledSelections.size === 0 && newGroups.length > 0 && wasGroupsEmpty) {
			const topGroups = newGroups.slice(0, maxDefaultSelected).map(g => g.label);
			return new Set(topGroups);
		} else {
			// Apply group limit when over the limit to prevent expensive computations
			if (reconciledSelections.size > maxDefaultSelected) {
				// Keep only the top groups by size when over the limit
				const sortedBySize = Array.from(reconciledSelections)
					.map(label => ({ label, size: newGroups.find(g => g.label === label)?.size || 0 }))
					.sort((a, b) => b.size - a.size)
					.slice(0, maxDefaultSelected)
					.map(g => g.label);
				return new Set(sortedBySize);
			} else {
				return reconciledSelections;
			}
		}
	}

	// Smart reconciliation: preserve selections using semantic matching
	// Only show toast when switching tabs, not when modifying fields within same tab
	let lastActiveTab = activeTab;
	let lastSelectedFields = new Set(selectedFields); // Track previous field selection
	let lastSecondaryFields = new Set<string>(); // Track previous secondary field selection
	async function reconcileGroups(newGroups: Array<{label: string, size: number}>) {
		const previousSelections = [...selectedGroups];
		const wasEmpty = groups.length === 0;
		
		// Use the generic reconciliation function
		const reconciledSelections = reconcileGroupSelections(
			previousSelections,
			newGroups,
			lastSelectedFields,
			selectedFields,
			wasEmpty
		);
		
		// Update state
		groups = newGroups;
		selectedGroups = reconciledSelections;
		
		// Calculate metrics for toast notifications
		const exactMatches = new Set(
			previousSelections.filter(selection => 
				newGroups.some(group => group.label === selection)
			)
		);
		
		// Try semantic matching for notification calculation
		let semanticMatches = exactMatches;
		if (exactMatches.size === 0 && previousSelections.length > 0 && !setsEqual(lastSelectedFields, selectedFields)) {
			const oldFieldCount = lastSelectedFields.size;
			const newFieldCount = selectedFields.size;
			
			if (newFieldCount > oldFieldCount) {
				semanticMatches = findChildGroups(previousSelections, newGroups, lastSelectedFields, selectedFields);
			} else if (newFieldCount < oldFieldCount) {
				semanticMatches = findParentGroups(previousSelections, newGroups, lastSelectedFields, selectedFields);
			} else {
				const childMatches = findChildGroups(previousSelections, newGroups, lastSelectedFields, selectedFields);
				const parentMatches = findParentGroups(previousSelections, newGroups, lastSelectedFields, selectedFields);
				semanticMatches = childMatches.size > 0 ? childMatches : parentMatches;
			}
		}
		
		// Only notify about lost selections if:
		// 1. There were previous selections
		// 2. Some were lost
		// 3. This is happening due to a tab change (not field modification)
		const totalLost = previousSelections.length - selectedGroups.size;
		const unavailableLost = previousSelections.length - (exactMatches.size + semanticMatches.size);
		const limitLost = semanticMatches.size - selectedGroups.size;
		const isTabChange = lastActiveTab !== activeTab;
		const isFieldChange = !setsEqual(lastSelectedFields, selectedFields);
		
		if (totalLost > 0 && previousSelections.length > 0 && isTabChange && !isFieldChange) {
			// Only show toasts for tab changes, not field changes (which should be seamless)
			if (unavailableLost > 0 && limitLost > 0) {
				toast.info(`${unavailableLost} selections not available in ${activeTab.toUpperCase()}, lost ${limitLost} for performance`);
			} else if (unavailableLost > 0) {
				toast.info(`${unavailableLost} selections not available in ${activeTab.toUpperCase()}`);
			} else if (limitLost > 0) {
				toast.info(`Selections limited to ${MAX_DEFAULT_SELECTED_GROUPS} on switch - lost ${limitLost}`);
			}
		}
		
		// Update tracking variables
		lastActiveTab = activeTab;
		lastSelectedFields = new Set(selectedFields);
		
		// Mark groups as reconciled for current field selection
		groupsReconciledForFields = new Set(selectedFields);
	}

	// Reactive: Load groups when fields change or tab switches
	$: if (data.length > 0 && !loading) {
		if (selectedFields.size > 0) {
			if (activeTab === 'pca') {
				loadPcaGroups();
			} else if (activeTab === 'ibd') {
				// Load groups for basic mode, or X-axis groups for cross-grouping mode
				loadIbdGroups();
			}
		} else {
			// Clear groups when no metadata fields are selected
			groups = [];
			selectedGroups = new Set();
		}
	}

	// Reactive: Load secondary-axis groups when cross-grouping mode is enabled and secondaryFields change
	$: if (crossGroupingMode) {
		if (secondaryFields.size > 0) {
			if (activeTab === 'pca') {
				loadPcaSecondaryGroups();
			} else if (activeTab === 'ibd') {
				loadSecondaryGroups();
			}
		} else {
			// Clear secondary groups when no secondary metadata fields are selected
			secondaryGroups = [];
			selectedSecondaryGroups = new Set();
			secondaryGroupsReconciledForFields = new Set();
		}
	}

	// Reactive: Generate available groups for Query dropdown
	$: availableQueryGroups = (() => {
		if (!groups.length) return [];
		
		// Filter groups by minimum size and format for display
		const MIN_QUERY_GROUP_SIZE = 30;
		const eligibleGroups = crossGroupingMode ? 
			// For cross-grouping mode, combine primary (main) and secondary groups
			[...new Set([...selectedGroups, ...selectedSecondaryGroups])]
				.map(label => {
					// Find group size from main groups or secondaryGroups
					const mainGroup = groups.find(g => g.label === label);
					const secondaryGroup = secondaryGroups.find(g => g.label === label);
					const size = mainGroup?.size || secondaryGroup?.size || 0;
					return { label, size };
				})
				.filter(group => group.size >= MIN_QUERY_GROUP_SIZE)
				.sort((a, b) => b.size - a.size)
				.map(group => `(${group.size}) ${group.label}`)
			:
			// For basic mode, use selected groups
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
				bind:secondaryFields
				bind:crossGroupingMode
				proposeCrossGrouping={true}
				title="Group by Metadata:"
				primaryFieldsLabel={activeTab === 'ibd' ? 'X Axis' : 'First Set'}
				secondaryFieldsLabel={activeTab === 'ibd' ? 'Y Axis' : 'Second Set'}
				on:fieldsChanged={handleFieldsChanged}
				on:secondaryFieldsChanged={handleSecondaryFieldsChanged}
				on:crossGroupingModeToggled={handleCrossGroupingModeToggled}
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
				{crossGroupingMode}
				{secondaryGroups}
				bind:selectedSecondaryGroups
				primaryFieldsLabel={activeTab === 'ibd' ? 'X Axis' : 'First Set'}
				secondaryFieldsLabel={activeTab === 'ibd' ? 'Y Axis' : 'Second Set'}
				on:groupsChanged={handleGroupsChanged}
				on:secondaryGroupsChanged={handleSecondaryGroupsChanged}
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
					bind:crossGroupingMode
					bind:groups
					bind:selectedGroups
					bind:secondaryFields
					bind:secondaryGroups
					bind:selectedSecondaryGroups
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
					bind:crossGroupingMode
					bind:groups
					bind:selectedGroups
					bind:secondaryFields
					bind:secondaryGroups
					bind:selectedSecondaryGroups
					loading={groupsLoading}
					{Plotly}
					isActive={activeTab === 'ibd'}
					{groupsReconciledForFields}
					{secondaryGroupsReconciledForFields}
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
