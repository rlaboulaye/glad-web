<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';
	import Legend from './Legend.svelte';
	import MetadataFieldSelector from './MetadataFieldSelector.svelte';
	import GroupSelector from './GroupSelector.svelte';

	// Props
	export let data: any[] = [];
	export let availableFields: string[];
	export let selectedFields: Set<string>;
	export let groups: Array<{label: string, size: number}> = [];
	export let selectedGroups: Set<string> = new Set();
	// Removed: loading is handled by parent component
	// export let loading: boolean = false;
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

	// Controlled color palette for predictable group colors
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

	// Track color assignments for consistency
	let groupColorMap = new Map<string, string>();
	
	// Track which groups have been rendered as traces
	let renderedTraces = new Map<string, number>(); // groupLabel -> traceIndex

	// Reactive legend data
	let legendData: Array<{label: string, color: string, size: number}> = [];

	// Update legend data reactively when groups or fields change
	$: {
		if (data.length > 0 && selectedGroups.size > 0 && selectedFields.size > 0) {
			legendData = getLegendData();
		} else {
			legendData = [];
		}
	}

	// Function to get consistent color for a group
	function getGroupColor(groupLabel: string): string {
		if (!groupColorMap.has(groupLabel)) {
			// Assign color based on stable hash of group label for consistency
			const hash = groupLabel.split('').reduce((a, b) => {
				a = ((a << 5) - a) + b.charCodeAt(0);
				return a & a;
			}, 4);
			const colorIndex = Math.abs(hash) % COLOR_PALETTE.length;
			groupColorMap.set(groupLabel, COLOR_PALETTE[colorIndex]);
			
		}
		return groupColorMap.get(groupLabel)!;
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
		const toAdd = [...selectedGroups].filter(group => !renderedTraces.has(group));
		
		// Find groups that need to be hidden (rendered but not selected)
		const toHide = [...renderedTraces.keys()].filter(group => !selectedGroups.has(group));
		
		// Find groups that need to be shown (rendered and selected)
		const toShow = [...selectedGroups].filter(group => renderedTraces.has(group));

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
			if (selectedGroups.has(groupLabel)) {
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
			selectedGroups.has(key)
		);

		const traces = filteredEntries.map(([key, group]) => ({
			x: group.map(d => d.pc[pcX]),
			y: group.map(d => d.pc[pcY]),
			text: group.map(d => key),
			mode: 'markers',
			type: 'scattergl',
			name: `(${group.length}) ${key}`,
			hovertemplate: `PC${pcX + 1}: %{x}<br>PC${pcY + 1}: %{y}<br>%{text}<extra></extra>`,
			marker: {
				size: 6,
				opacity: 0.6,
				color: getGroupColor(key),
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


	// Handle field changes from parent
	export function onFieldsChanged() {
		// Clear render optimizations when fields change (groups managed externally now)
		renderedTraces = new Map(); // Clear trace tracking
		groupColorMap = new Map(); // Clear color assignments for fresh mapping
		plotInitialized = false; // Force re-render
	}

	// Event handler for MetadataFieldSelector component
	function handleFieldsChanged(event) {
		selectedFields = event.detail;
		onFieldsChanged();
	}

	// Event handlers for GroupSelector component
	function handleGroupsChanged(event) {
		selectedGroups = event.detail;
		updatePlot();
	}

	function handleSelectAll() {
		selectedGroups = new Set(groups.map(g => g.label));
		updatePlot();
	}

	function handleDeselectAll() {
		selectedGroups = new Set();
		updatePlot();
	}


	// Get available groups for query dropdown (filtered by minimum size and only selected groups)
	export function getAvailableQueryGroups() {
		if (!data.length || selectedGroups.size === 0) return [];
		
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
			.filter(([key, group]) => selectedGroups.has(key) && group.length >= MIN_QUERY_GROUP_SIZE)
			.sort((a, b) => b[1].length - a[1].length)
			.map(([key, group]) => `(${group.length}) ${key}`);
	}

	// Get legend data for visible groups with colors and sizes
	function getLegendData(): Array<{label: string, color: string, size: number}> {
		if (!data.length || selectedGroups.size === 0) return [];
		
		const orderedFields = availableFields.filter(f => selectedFields.has(f));
		const grouped = new Map<string, any[]>();
		
		for (const d of data) {
			const key = orderedFields.length > 0
				? orderedFields.map(f => d[f] ?? 'Unknown').join(' | ')
				: 'All';
			if (!grouped.has(key)) grouped.set(key, []);
			grouped.get(key)!.push(d);
		}
		
		// Return only selected groups with their colors and sizes
		return Array.from(grouped.entries())
			.filter(([key, group]) => selectedGroups.has(key))
			.sort((a, b) => b[1].length - a[1].length)
			.map(([key, group]) => ({
				label: key,
				color: getGroupColor(key),
				size: group.length
			}));
	}

	// Handle PCA tab activation and plot rendering
	$: if (isActive && data.length > 0 && Plotly && plotDiv && selectedFields.size > 0 && groups.length > 0 && selectedGroups.size > 0) {
		// Groups are loaded and selected, render plot
		if (!plotInitialized) {
			setTimeout(() => updatePlot(), 100);
		}
	}

	// Reset plot initialization when tab becomes active (handles tab switching)
	$: if (isActive && groups.length > 0 && selectedGroups.size > 0) {
		plotInitialized = false;
	}

	// Render plot when groups are loaded and selected (handles initial load)
	$: if (isActive && groups.length > 0 && selectedGroups.size > 0 && Plotly && plotDiv && data.length > 0) {
		// Only render if plot hasn't been initialized
		if (!plotInitialized) {
			setTimeout(() => updatePlot(), 100);
		}
	}

	// Clear plot when no metadata fields are selected
	$: if (isActive && selectedFields.size === 0 && Plotly && plotDiv) {
		Plotly.purge(plotDiv);
		plotInitialized = false;
		renderedTraces = new Map();
		groupColorMap = new Map();
	}
</script>


<!-- Note: MetadataFieldSelector and GroupSelector are now handled by parent -->

<!-- PCA Plot container with integrated legend -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="flex h-[600px] md:h-[700px] lg:h-[750px] gap-4">
		<!-- Plot area -->
		<div class="flex-1 relative">
			{#if selectedGroups.size === 0}
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
		
		<!-- Legend sidebar -->
		{#if legendData.length > 0}
			<div class="w-48 flex-shrink-0">
				<div class="h-full flex flex-col">
					<!-- Axis selectors -->
					<div class="mb-4 space-y-3">
						<label class="block text-gray-700 dark:text-gray-300 text-sm font-semibold">
							X Axis:
							<select 
								bind:value={pcX} 
								on:change={updatePlot} 
								class="mt-1 w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 text-sm"
							>
								{#each Array(10).fill(0).map((_, i) => i) as i}
									<option value={i}>PC{i + 1}</option>
								{/each}
							</select>
						</label>

						<label class="block text-gray-700 dark:text-gray-300 text-sm font-semibold">
							Y Axis:
							<select 
								bind:value={pcY} 
								on:change={updatePlot} 
								class="mt-1 w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 text-sm"
							>
								{#each Array(10).fill(0).map((_, i) => i) as i}
									<option value={i}>PC{i + 1}</option>
								{/each}
							</select>
						</label>
					</div>

					<div class="mb-3">
						<h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">Legend</h3>
						<p class="text-xs text-gray-600 dark:text-gray-400 mt-1">
							{legendData.length} groups
						</p>
					</div>
					
					<div class="flex-1 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-3">
						<div class="space-y-1">
							{#each legendData as group}
								<div 
									class="flex items-center space-x-2 py-1 cursor-default"
									title="{group.label} ({group.size} individuals)"
								>
									<!-- Color swatch -->
									<div 
										class="w-3 h-3 rounded-full border border-gray-300 dark:border-gray-500 flex-shrink-0"
										style="background-color: {group.color}"
									></div>
									
									<!-- Group label and count -->
									<div class="flex-1 min-w-0">
										<div class="text-xs text-gray-700 dark:text-gray-300 truncate leading-tight">
											{group.label}
											<span class="text-gray-500 dark:text-gray-400">
												({group.size})
											</span>
										</div>
									</div>
								</div>
							{/each}
						</div>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>
