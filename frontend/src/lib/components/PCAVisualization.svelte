<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';

	// Props
	export let data: any[] = [];
	export let availableFields: string[];
	export let selectedFields: Set<string>;
	export let Plotly: any = null;
	export let isActive: boolean = false;

	// Internal state
	let plotDiv: HTMLDivElement;
	let pcX = 0;
	let pcY = 1;
	
	// Track current plot state to optimize updates
	let currentPcX = -1;
	let currentPcY = -1;
	let currentSelectedFields = new Set<string>();
	let plotInitialized = false;
	
	// PCA group selection
	let pcaGroups: Array<{label: string, size: number}> = [];
	let selectedPcaGroups = new Set<string>();
	let pcaGroupsLoading = false;
	
	// Track which groups have been rendered as traces
	let renderedTraces = new Map<string, number>(); // groupLabel -> traceIndex

	// PCA Group Selection Functions
	function loadPcaGroups() {
		if (!data.length) return;
		
		pcaGroupsLoading = true;
		
		try {
			// Generate groups from current data and field selection
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
			pcaGroups = Array.from(grouped.entries())
				.map(([label, individuals]) => ({ label, size: individuals.length }))
				.sort((a, b) => b.size - a.size);

			// Pre-select top 16 groups for default view
			const topGroups = pcaGroups.slice(0, 16).map(g => g.label);
			selectedPcaGroups = new Set(topGroups);
			
		} catch (err) {
			toast.error('Failed to load PCA groups');
			console.error('Error loading PCA groups:', err);
		} finally {
			pcaGroupsLoading = false;
		}
	}

	function togglePcaGroup(groupLabel: string) {
		if (selectedPcaGroups.has(groupLabel)) {
			selectedPcaGroups = new Set([...selectedPcaGroups].filter(g => g !== groupLabel));
		} else {
			selectedPcaGroups = new Set([groupLabel, ...selectedPcaGroups]);
		}
		updatePlot();
	}

	function selectAllPcaGroups() {
		selectedPcaGroups = new Set(pcaGroups.map(g => g.label));
		updatePlot();
	}

	function deselectAllPcaGroups() {
		selectedPcaGroups = new Set();
		updatePlot();
	}

	function updatePlot() {
		if (!data.length || !Plotly || !plotDiv) return;

		// Always order selectedFields by their order in availableFields
		const orderedFields = availableFields.filter(f => selectedFields.has(f));

		// Check if we need full re-render or can use partial update
		const fieldsChanged = !setsEqual(selectedFields, currentSelectedFields);
		const axesChanged = pcX !== currentPcX || pcY !== currentPcY;
		const needsFullRender = !plotInitialized || fieldsChanged;

		if (needsFullRender) {
			// Full re-render needed - grouping changed, clear trace tracking
			renderedTraces = new Map();
			createFullPlot(orderedFields);
		} else if (axesChanged) {
			// Only axes changed - use restyle for better performance
			updatePlotAxes(orderedFields);
		} else {
			// Hybrid approach: handle group selection changes
			handleGroupSelectionChanges(orderedFields);
		}
		
		// Update tracking variables
		currentPcX = pcX;
		currentPcY = pcY;
		currentSelectedFields = new Set(selectedFields);
		plotInitialized = true;
	}

	function handleGroupSelectionChanges(orderedFields: string[]) {
		// Find groups that need to be added (new selections not yet rendered)
		const toAdd = [...selectedPcaGroups].filter(group => !renderedTraces.has(group));
		
		// Find groups that need to be hidden (rendered but not selected)
		const toHide = [...renderedTraces.keys()].filter(group => !selectedPcaGroups.has(group));
		
		// Find groups that need to be shown (rendered and selected)
		const toShow = [...selectedPcaGroups].filter(group => renderedTraces.has(group));

		if (toAdd.length > 0) {
			// Need to add new traces - full re-render required
			renderedTraces = new Map();
			createFullPlot(orderedFields);
		} else if (toHide.length > 0 || toShow.length > 0) {
			// Just toggle visibility of existing traces
			toggleTraceVisibility(toShow, toHide);
		}
	}

	function toggleTraceVisibility(toShow: string[], toHide: string[]) {
		// Create visibility array for all current traces
		const visibilityUpdates: boolean[] = [];
		
		// Build visibility array based on trace order
		for (const [groupLabel, traceIndex] of renderedTraces) {
			if (selectedPcaGroups.has(groupLabel)) {
				visibilityUpdates[traceIndex] = true;
			} else {
				visibilityUpdates[traceIndex] = false;
			}
		}

		// Apply visibility changes
		Plotly.restyle(plotDiv, { visible: visibilityUpdates });
	}

	function createFullPlot(orderedFields: string[]) {
		const { traces, layout } = generatePlotData(orderedFields);
		
		// Update trace tracking map
		renderedTraces.clear();
		traces.forEach((trace, index) => {
			// Extract group label from trace name (remove count prefix)
			const groupLabel = trace.name.replace(/^\(\d+\)\s/, '');
			renderedTraces.set(groupLabel, index);
		});
		
		Plotly.newPlot(plotDiv, traces, layout, {
			responsive: true,
			displayModeBar: true,
			modeBarButtonsToRemove: ['pan2d', 'lasso2d', 'select2d', 'autoScale2d'],
			displaylogo: false
		});
	}

	function updatePlotAxes(orderedFields: string[]) {
		const { traces, layout } = generatePlotData(orderedFields);
		
		// Update just the data and axes using restyle/relayout for better performance
		const updateData = {
			x: traces.map(trace => trace.x),
			y: traces.map(trace => trace.y)
		};
		
		const updateLayout = {
			'title.text': layout.title.text,
			'xaxis.title': layout.xaxis.title,
			'yaxis.title': layout.yaxis.title
		};
		
		// Use Promise.all for concurrent updates
		Promise.all([
			Plotly.restyle(plotDiv, updateData),
			Plotly.relayout(plotDiv, updateLayout)
		]);
	}

	function generatePlotData(orderedFields: string[]) {
		const grouped = new Map<string, any[]>();

		for (const d of data) {
			const key = orderedFields.length > 0
				? orderedFields.map(f => d[f] ?? 'Unknown').join(' | ')
				: 'All';

			if (!grouped.has(key)) grouped.set(key, []);
			grouped.get(key)!.push(d);
		}

		const sortedEntries = Array.from(grouped.entries()).sort(
			(a, b) => b[1].length - a[1].length
		);

		// Filter traces to only include selected groups
		const filteredEntries = sortedEntries.filter(([key, group]) => 
			selectedPcaGroups.has(key)
		);

		const traces = filteredEntries.map(([key, group]) => ({
			x: group.map(d => d.pc[pcX]),
			y: group.map(d => d.pc[pcY]),
			text: group.map(d => d.id),
			mode: 'markers',
			type: 'scattergl',
			name: `(${group.length}) ${key}`,
			marker: {
				size: 6,
				opacity: 0.6,
			},
		}));

		const layout = {
			title: {
				text: `PCA: PC${pcX + 1} vs PC${pcY + 1}`,
				font: {
					size: 20,
					color: '#1f2937'
				}
			},
			autosize: true,
			xaxis: {
				title: `PC${pcX + 1}`,
				automargin: true,
				zeroline: false,
				gridcolor: '#f3f4f6',
				linecolor: '#d1d5db'
			},
			yaxis: {
				title: `PC${pcY + 1}`,
				automargin: true,
				zeroline: false,
				gridcolor: '#f3f4f6',
				linecolor: '#d1d5db'
			},
			margin: { t: 60, l: 60, r: 80, b: 60 },
			hovermode: 'closest',
			plot_bgcolor: '#ffffff',
			paper_bgcolor: '#ffffff',
			font: {
				family: 'system-ui, -apple-system, sans-serif',
				size: 12,
				color: '#374151'
			},
			showlegend: false
		};

		return { traces, layout };
	}

	// Helper function to compare sets for equality
	function setsEqual<T>(a: Set<T>, b: Set<T>): boolean {
		return a.size === b.size && [...a].every(x => b.has(x));
	}

	// Get tooltip text for metadata fields
	function getFieldTooltip(field: string): string {
		switch (field) {
			case 'phs':
				return 'Unique identifier assigned to a phenotype study by dbGaP';
			case 'country':
				return 'Country where the sample was taken';
			case 'region':
				return 'Region where the sample was taken';
			case 'sex':
				return 'Reported sex of sample';
			case 'ethnicity':
				return 'One of ["Hispanic", "NotHispanic", "NativeAmerican"]';
			case 'self_described':
				return 'True if sample self-reported as "Hispanic"';
			case 'ibd_community':
				return 'Community assignment produced by running Infomap on total pairwise IBD between samples';
			default:
				return '';
		}
	}

	// Handle field changes from parent
	export function onFieldsChanged() {
		// Reset PCA groups when fields change
		pcaGroups = [];
		selectedPcaGroups = new Set();
		renderedTraces = new Map(); // Clear trace tracking
		if (selectedFields.size > 0 && isActive && data.length > 0) {
			loadPcaGroups();
		}
	}

	// Reactive export to trigger parent updates when selections change
	export let queryGroupsUpdateTrigger = 0;
	
	// Trigger parent update when PCA groups selection changes
	$: if (selectedPcaGroups) {
		queryGroupsUpdateTrigger = Date.now();
	}

	// Get available groups for query dropdown (filtered by minimum size and only selected groups)
	export function getAvailableQueryGroups() {
		if (!data.length || selectedPcaGroups.size === 0) return [];
		
		const orderedFields = availableFields.filter(f => selectedFields.has(f));
		const grouped = new Map<string, any[]>();
		
		for (const d of data) {
			const key = orderedFields.length > 0
				? orderedFields.map(f => d[f] ?? 'Unknown').join(' | ')
				: 'All';
			if (!grouped.has(key)) grouped.set(key, []);
			grouped.get(key)!.push(d);
		}
		
		// Return only selected groups that meet minimum size requirement (30 individuals)
		// Sorted by size (descending) with group name and count
		const MIN_QUERY_GROUP_SIZE = 30;
		return Array.from(grouped.entries())
			.filter(([key, group]) => selectedPcaGroups.has(key) && group.length >= MIN_QUERY_GROUP_SIZE)
			.sort((a, b) => b[1].length - a[1].length)
			.map(([key, group]) => `(${group.length}) ${key}`);
	}

	// Handle PCA tab activation and plot rendering
	$: if (isActive && data.length > 0 && Plotly && plotDiv) {
		if (selectedFields.size > 0) {
			if (pcaGroups.length === 0) {
				// Need to load groups first
				loadPcaGroups();
			} else if (selectedPcaGroups.size > 0) {
				// Groups are loaded and selected, render plot
				// Force re-render by resetting plotInitialized flag
				plotInitialized = false;
				setTimeout(() => updatePlot(), 100);
			}
		}
	}

	// Reset plot initialization when tab becomes active (handles tab switching)
	$: if (isActive && pcaGroups.length > 0 && selectedPcaGroups.size > 0) {
		plotInitialized = false;
	}

	// Render plot when groups are loaded and selected (handles initial load)
	$: if (isActive && pcaGroups.length > 0 && selectedPcaGroups.size > 0 && Plotly && plotDiv && data.length > 0) {
		// Only render if plot hasn't been initialized
		if (!plotInitialized) {
			setTimeout(() => updatePlot(), 100);
		}
	}
</script>

<!-- PCA Controls -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="space-y-6">
		<!-- Axis selectors -->
		<div class="flex flex-wrap gap-6 items-center">
			<label class="text-gray-700 dark:text-gray-300 font-semibold">
				X Axis:
				<select 
					bind:value={pcX} 
					on:change={updatePlot} 
					class="ml-2 p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
				>
					{#each Array(10).fill(0).map((_, i) => i) as i}
						<option value={i}>PC{i + 1}</option>
					{/each}
				</select>
			</label>

			<label class="text-gray-700 dark:text-gray-300 font-semibold">
				Y Axis:
				<select 
					bind:value={pcY} 
					on:change={updatePlot} 
					class="ml-2 p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
				>
					{#each Array(10).fill(0).map((_, i) => i) as i}
						<option value={i}>PC{i + 1}</option>
					{/each}
				</select>
			</label>
		</div>

		<!-- Metadata fields selector -->
		<div>
			<p class="text-gray-700 dark:text-gray-300 mb-3 font-semibold">Color and Filter by Metadata Fields:</p>
			<div class="flex flex-wrap gap-2">
				{#each availableFields as field}
					<button
						class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
							{selectedFields.has(field)
								? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
								: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
						title={getFieldTooltip(field)}
						on:click={() => {
							if (selectedFields.has(field)) {
								selectedFields.delete(field);
							} else {
								selectedFields.add(field);
							}
							selectedFields = selectedFields;
							onFieldsChanged();
						}}
					>
						{field}
					</button>
				{/each}
			</div>
		</div>
	</div>
</div>

<!-- PCA Group Selection -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="space-y-6">
		{#if selectedFields.size === 0}
			<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
				<div class="text-center">
					<div class="text-3xl mb-2">🏷️</div>
					<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Select Metadata Fields</h3>
					<p class="text-gray-600 dark:text-gray-400">
						Choose metadata fields above to group the data
					</p>
				</div>
			</div>
		{:else}
			<!-- Group Selection -->
			<div>
				<div class="flex items-center justify-between mb-4">
					<div>
						<p class="text-gray-700 dark:text-gray-300 font-semibold">Group Selection:</p>
						<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
							Choose groups to display in the PCA plot ({selectedPcaGroups.size} selected)
						</p>
					</div>
					<div class="flex items-center space-x-3">
						{#if pcaGroups.length > 0}
							<button
								class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
								on:click={selectAllPcaGroups}
							>
								Select All
							</button>
							<button
								class="px-3 py-1.5 text-sm font-medium transition-colors duration-200 rounded-md
									{selectedPcaGroups.size > 0 
										? 'text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 hover:bg-gray-200 dark:hover:bg-gray-500' 
										: 'text-gray-400 dark:text-gray-500 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 cursor-not-allowed'}"
								on:click={deselectAllPcaGroups}
								disabled={selectedPcaGroups.size === 0}
							>
								Deselect All
							</button>
						{/if}
					</div>
				</div>

				{#if pcaGroups.length > 0}
					<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
						<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
							{#each pcaGroups as group}
								<button
									class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
										{selectedPcaGroups.has(group.label)
											? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
											: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
									on:click={() => togglePcaGroup(group.label)}
									disabled={pcaGroupsLoading}
								>
									<div class="font-medium">{group.label}</div>
									<div class="text-xs opacity-75">{group.size} individuals</div>
								</button>
							{/each}
						</div>
					</div>
				{:else if pcaGroupsLoading}
					<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
						<div class="text-center">
							<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							<p class="text-sm text-gray-600 dark:text-gray-400">Loading groups...</p>
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<!-- PCA Plot container -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="w-full h-[600px] md:h-[700px] lg:h-[750px] relative">
		{#if selectedPcaGroups.size === 0}
			<div class="absolute inset-0 flex items-center justify-center bg-white dark:bg-gray-800 z-10">
				<div class="text-center">
					<div class="text-4xl mb-4">📊</div>
					<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Select Groups</h3>
					<p class="text-gray-600 dark:text-gray-400">
						Choose groups above to display in the PCA plot
					</p>
				</div>
			</div>
		{/if}
		<div bind:this={plotDiv} class="w-full h-full"></div>
	</div>
</div>