<script lang="ts">
	import { toast } from '$lib/toast.js';
	import MetadataFieldSelector from './MetadataFieldSelector.svelte';
	import GroupSelector from './GroupSelector.svelte';

	// Props
	export let availableFields: string[];
	export let selectedFields: Set<string>; // Shared field selections like PCA
	export let Plotly: any = null;
	export let isActive: boolean = false;

	// IBD grouping fields (asymmetric mode only - symmetric uses shared selectedFields)
	let asymmetricMode = false; // Toggle for asymmetric heatmap
	let ibdXFields = new Set(['ibd_community']); // X-axis fields for asymmetric mode
	let ibdYFields = new Set(['ibd_community']); // Y-axis fields for asymmetric mode
	
	// IBD Heatmap data
	let ibdGroups: Array<{label: string, size: number}> = [];
	let selectedGroups = new Set<string>();
	let heatmapDiv: HTMLDivElement;
	let ibdLoading = false;
	let plotRendering = false;
	let heatmapData: any = null;
	let logScale = false; // Toggle for log scale visualization
	
	// IBD Groups for asymmetric mode
	let ibdXGroups: Array<{label: string, size: number}> = [];
	let ibdYGroups: Array<{label: string, size: number}> = [];
	let selectedXGroups = new Set<string>();
	let selectedYGroups = new Set<string>();


	// Event handlers for MetadataFieldSelector component
	function handleFieldsChanged(event) {
		selectedFields = event.detail;
		// Reset groups when grouping changes
		ibdGroups = [];
		selectedGroups = new Set();
		if (selectedFields.size > 0) {
			loadIbdGroups();
		}
	}

	function handleXFieldsChanged(event) {
		ibdXFields = event.detail;
		loadAsymmetricGroups();
	}

	function handleYFieldsChanged(event) {
		ibdYFields = event.detail;
		loadAsymmetricGroups();
	}

	function handleAsymmetricModeToggled(event) {
		asymmetricMode = event.detail;
		
		if (asymmetricMode) {
			// Copy current symmetric fields to both X and Y
			ibdXFields = new Set(selectedFields);
			ibdYFields = new Set(selectedFields);
			// Reset groups
			ibdXGroups = [];
			ibdYGroups = [];
			selectedXGroups = new Set();
			selectedYGroups = new Set();
			loadAsymmetricGroups();
		} else {
			// Copy current X axis fields to symmetric mode
			selectedFields = new Set(ibdXFields);
			selectedFields = selectedFields; // Trigger reactivity
			// Reset groups
			ibdGroups = [];
			selectedGroups = new Set();
			if (selectedFields.size > 0) {
				loadIbdGroups();
			}
		}
	}

	// Event handlers for GroupSelector component
	function handleGroupsChanged(event) {
		selectedGroups = event.detail;
		updateHeatmap();
	}

	function handleXGroupsChanged(event) {
		selectedXGroups = event.detail;
		updateAsymmetricHeatmap();
	}

	function handleYGroupsChanged(event) {
		selectedYGroups = event.detail;
		updateAsymmetricHeatmap();
	}

	function handleDeselectAllGroups() {
		selectedGroups = new Set();
	}

	function handleDeselectAllXGroups() {
		selectedXGroups = new Set();
	}

	function handleDeselectAllYGroups() {
		selectedYGroups = new Set();
	}

	// IBD Heatmap functions

	async function loadIbdGroups() {
		// Don't load if no fields are selected
		if (selectedFields.size === 0) return;
		
		try {
			// Generate grouping parameter from selected fields
			const orderedFields = availableFields.filter(f => selectedFields.has(f));
			const grouping = orderedFields.join(',');
			
			const res = await fetch(`/api/ibd-groups?grouping=${encodeURIComponent(grouping)}&min_size=30`);
			if (!res.ok) {
				throw new Error('Failed to load IBD groups');
			}
			const data = await res.json();
			ibdGroups = data.groups;
			
			// Pre-select top 16 groups for default view
			const topGroups = ibdGroups.slice(0, 16).map(g => g.label);
			selectedGroups = new Set(topGroups);
			
		} catch (err) {
			toast.error('Failed to load IBD communities');
			console.error('Error loading IBD groups:', err);
		}
	}

	async function loadAsymmetricGroups() {
		// Load both X and Y groups in parallel
		const promises = [];
		
		if (ibdXFields.size > 0) {
			const xOrderedFields = availableFields.filter(f => ibdXFields.has(f));
			const xGrouping = xOrderedFields.join(',');
			promises.push(
				fetch(`/api/ibd-groups?grouping=${encodeURIComponent(xGrouping)}&min_size=30`)
					.then(res => res.json())
					.then(data => ({ axis: 'x', groups: data.groups }))
			);
		}
		
		if (ibdYFields.size > 0) {
			const yOrderedFields = availableFields.filter(f => ibdYFields.has(f));
			const yGrouping = yOrderedFields.join(',');
			promises.push(
				fetch(`/api/ibd-groups?grouping=${encodeURIComponent(yGrouping)}&min_size=30`)
					.then(res => res.json())
					.then(data => ({ axis: 'y', groups: data.groups }))
			);
		}
		
		try {
			const results = await Promise.all(promises);
			
			for (const result of results) {
				if (result.axis === 'x') {
					ibdXGroups = result.groups;
					// Pre-select some groups for X
					const topXGroups = ibdXGroups.slice(0, 8).map(g => g.label);
					selectedXGroups = new Set(topXGroups);
				} else {
					ibdYGroups = result.groups;
					// Pre-select some groups for Y
					const topYGroups = ibdYGroups.slice(0, 8).map(g => g.label);
					selectedYGroups = new Set(topYGroups);
				}
			}
		} catch (err) {
			toast.error('Failed to load IBD groups');
			console.error('Error loading asymmetric IBD groups:', err);
		}
	}

	async function updateHeatmap() {
		if (!selectedGroups.size || !Plotly || !heatmapDiv || selectedFields.size === 0) return;

		ibdLoading = true;
		try {
			// Generate grouping parameter from selected fields
			const orderedFields = availableFields.filter(f => selectedFields.has(f));
			const grouping = orderedFields.join(',');
			
			const res = await fetch('/api/ibd-matrix', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					grouping: grouping,
					selected_groups: [...selectedGroups]
				})
			});

			if (!res.ok) {
				throw new Error('Failed to compute IBD matrix');
			}

			heatmapData = await res.json();
			await renderHeatmap();

		} catch (err) {
			toast.error('Failed to compute IBD matrix');
			console.error('Error computing IBD matrix:', err);
		} finally {
			ibdLoading = false;
		}
	}

	async function updateAsymmetricHeatmap() {
		if (!selectedXGroups.size || !selectedYGroups.size || !Plotly || !heatmapDiv || 
			ibdXFields.size === 0 || ibdYFields.size === 0) return;

		ibdLoading = true;
		try {
			// Generate grouping parameters from selected fields
			const xOrderedFields = availableFields.filter(f => ibdXFields.has(f));
			const yOrderedFields = availableFields.filter(f => ibdYFields.has(f));
			const xGrouping = xOrderedFields.join(',');
			const yGrouping = yOrderedFields.join(',');
			
			const res = await fetch('/api/ibd-matrix-asymmetric', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					row_grouping: yGrouping,      // Y Axis → rows  
					column_grouping: xGrouping,   // X Axis → columns
					selected_row_groups: [...selectedYGroups],    // Y Axis → rows
					selected_column_groups: [...selectedXGroups] // X Axis → columns
				})
			});

			if (!res.ok) {
				throw new Error('Failed to compute asymmetric IBD matrix');
			}

			heatmapData = await res.json();
			await renderHeatmap();

		} catch (err) {
			toast.error('Failed to compute asymmetric IBD matrix');
			console.error('Error computing asymmetric IBD matrix:', err);
		} finally {
			ibdLoading = false;
		}
	}

	async function renderHeatmap() {
		if (!heatmapData || !Plotly || !heatmapDiv) return;

		plotRendering = true;

		// Handle both symmetric and asymmetric data
		const isAsymmetric = heatmapData.row_group_labels && heatmapData.row_group_sizes;
		const { matrix } = heatmapData;
		
		let xLabels, yLabels, processedMatrix, matrixTitle;
		
		if (isAsymmetric) {
			// Asymmetric mode: columns are X axis, rows are Y axis
			const { row_group_labels, row_group_sizes, group_labels: column_group_labels, group_sizes: column_group_sizes } = heatmapData;
			
			// Create labels with sizes
			xLabels = column_group_labels.map((label, i) => `${label} (${column_group_sizes[i]})`);
			yLabels = row_group_labels.map((label, i) => `${label} (${row_group_sizes[i]})`).reverse();
			
			// Apply log transformation and reverse matrix rows to match y-axis ordering
			processedMatrix = logScale 
				? matrix.map(row => row.map(val => val > 0 ? Math.log10(val + 0.001) : Math.log10(0.001))).reverse()
				: matrix.slice().reverse();
			
			matrixTitle = `IBD Heatmap: ${row_group_labels.length} × ${column_group_labels.length} Groups`;
		} else {
			// Symmetric mode: same logic as before
			const { group_labels, group_sizes } = heatmapData;
			
			// Create sorted indices for y-axis (descending by size)
			const sortedIndices = group_labels
				.map((label, i) => ({ index: i, size: group_sizes[i], label }))
				.sort((a, b) => b.size - a.size)
				.map(item => item.index);

			// Conditionally apply log transformation based on toggle
			const logProcessedMatrix = logScale 
				? matrix.map(row => row.map(val => val > 0 ? Math.log10(val + 0.001) : Math.log10(0.001)))
				: matrix;

			// Reorder matrix rows and y-labels by descending size
			processedMatrix = sortedIndices.map(i => logProcessedMatrix[i]).reverse();
			yLabels = sortedIndices.map(i => `${group_labels[i]} (${group_sizes[i]})`).reverse();
			xLabels = group_labels.map((label, i) => `${label} (${group_sizes[i]})`);
			
			matrixTitle = `IBD Heatmap: ${group_labels.length} Communities`;
		}

		const trace = {
			z: processedMatrix,
			x: xLabels,
			y: yLabels,
			type: 'heatmap',
			showscale: true,
			hoverongaps: false,
			colorbar: {
				title: logScale ? 'Mean IBD (log₁₀)' : 'Mean IBD',
				titleside: 'right'
			},
			colorscale: 'Greens',
			reversescale: true // Dark green = high values
		};

		const layout = {
			title: {
				text: matrixTitle,
				font: { size: 20, color: '#1f2937' }
			},
			autosize: true,
			xaxis: {
				title: isAsymmetric ? `X Axis Groups (${heatmapData.column_grouping})` : 'IBD Community (Size)',
				side: 'bottom',
				tickangle: -45,
				automargin: true
			},
			yaxis: {
				title: isAsymmetric ? `Y Axis Groups (${heatmapData.row_grouping})` : 'IBD Community (Size)',
				side: 'left',
				automargin: true
			},
			margin: { t: 80, l: 120, r: 80, b: 120 },
			plot_bgcolor: '#ffffff',
			paper_bgcolor: '#ffffff',
			font: {
				family: 'system-ui, -apple-system, sans-serif',
				size: 10,
				color: '#374151'
			}
		};

		try {
			await Plotly.newPlot(heatmapDiv, [trace], layout, {
				responsive: true,
				displayModeBar: true,
				displaylogo: false
			});
		} finally {
			plotRendering = false;
		}
	}


	// Reactive export to trigger parent updates when selections change
	export let queryGroupsUpdateTrigger = 0;
	
	// Trigger parent update when IBD groups selection changes
	$: if (selectedGroups || selectedXGroups || selectedYGroups || asymmetricMode) {
		queryGroupsUpdateTrigger = Date.now();
	}

	// Get available groups for query dropdown (filtered by minimum size)
	export function getAvailableQueryGroups() {
		const MIN_QUERY_GROUP_SIZE = 30;
		
		if (asymmetricMode) {
			// Combine X and Y groups, remove duplicates, filter by size, sort by size
			const allGroups = new Set([...selectedXGroups, ...selectedYGroups]);
			const groupsArray = Array.from(allGroups);
			
			// Create a map to get group sizes for filtering and sorting
			const groupSizeMap = new Map();
			ibdXGroups.forEach(g => groupSizeMap.set(g.label, g.size));
			ibdYGroups.forEach(g => groupSizeMap.set(g.label, g.size));
			
			return groupsArray
				.filter(groupLabel => {
					const size = groupSizeMap.get(groupLabel) || 0;
					return size >= MIN_QUERY_GROUP_SIZE;
				})
				.sort((a, b) => {
					const sizeA = groupSizeMap.get(a) || 0;
					const sizeB = groupSizeMap.get(b) || 0;
					return sizeB - sizeA; // Sort by size descending
				})
				.map(groupLabel => {
					const size = groupSizeMap.get(groupLabel) || 0;
					return `(${size}) ${groupLabel}`;
				});
		} else {
			// Use symmetric selected groups, filter by size, sort by size
			const groupsArray = Array.from(selectedGroups);
			const groupSizeMap = new Map(ibdGroups.map(g => [g.label, g.size]));
			
			return groupsArray
				.filter(groupLabel => {
					const size = groupSizeMap.get(groupLabel) || 0;
					return size >= MIN_QUERY_GROUP_SIZE;
				})
				.sort((a, b) => {
					const sizeA = groupSizeMap.get(a) || 0;
					const sizeB = groupSizeMap.get(b) || 0;
					return sizeB - sizeA; // Sort by size descending
				})
				.map(groupLabel => {
					const size = groupSizeMap.get(groupLabel) || 0;
					return `(${size}) ${groupLabel}`;
				});
		}
	}

	// Load IBD groups when tab becomes active (only if not loaded yet - for persistence)
	$: if (isActive && !asymmetricMode && ibdGroups.length === 0 && selectedFields.size > 0) {
		loadIbdGroups();
	}

	// Load asymmetric groups when tab becomes active in asymmetric mode (only if not loaded yet - for persistence)  
	$: if (isActive && asymmetricMode && (ibdXGroups.length === 0 || ibdYGroups.length === 0) && (ibdXFields.size > 0 || ibdYFields.size > 0)) {
		loadAsymmetricGroups();
	}

	// Update symmetric heatmap when selection or log scale changes (with persistence)
	$: if (isActive && !asymmetricMode && selectedGroups.size > 0 && Plotly && heatmapDiv && ibdGroups.length > 0) {
		setTimeout(() => updateHeatmap(), 100);
	}

	// Update asymmetric heatmap when selection changes (with persistence)
	$: if (isActive && asymmetricMode && selectedXGroups.size > 0 && selectedYGroups.size > 0 && Plotly && heatmapDiv && ibdXGroups.length > 0 && ibdYGroups.length > 0) {
		setTimeout(() => updateAsymmetricHeatmap(), 100);
	}
	
	// Re-render heatmap when log scale toggle changes
	$: if (isActive && heatmapData && Plotly && heatmapDiv && logScale !== undefined) {
		setTimeout(async () => await renderHeatmap(), 100);
	}
</script>

<!-- IBD Grouping Fields Selector -->
<MetadataFieldSelector 
	{availableFields}
	bind:selectedFields
	bind:xFields={ibdXFields}
	bind:yFields={ibdYFields}
	bind:asymmetricMode
	proposeAsymmetric={true}
	on:fieldsChanged={handleFieldsChanged}
	on:xFieldsChanged={handleXFieldsChanged}
	on:yFieldsChanged={handleYFieldsChanged}
	on:asymmetricModeToggled={handleAsymmetricModeToggled}
/>

<!-- IBD Group Selection -->
<GroupSelector 
	groups={ibdGroups}
	bind:selectedGroups
	loading={ibdLoading}
	description="Choose groups to visualize Identity-by-Descent patterns"
	enableSelectAll={false}
	bind:asymmetricMode
	xGroups={ibdXGroups}
	yGroups={ibdYGroups}
	bind:selectedXGroups
	bind:selectedYGroups
	on:groupsChanged={handleGroupsChanged}
	on:xGroupsChanged={handleXGroupsChanged}
	on:yGroupsChanged={handleYGroupsChanged}
	on:deselectAll={handleDeselectAllGroups}
/>

<!-- IBD Heatmap container -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<!-- Log Scale Toggle -->
	{#if ((!asymmetricMode && selectedGroups.size > 0) || (asymmetricMode && selectedXGroups.size > 0 && selectedYGroups.size > 0))}
		<div class="flex items-center justify-between mb-6 pb-4 border-b border-gray-200 dark:border-gray-600">
			<div>
				<p class="text-gray-700 dark:text-gray-300 font-medium">Visualization Scale</p>
				<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
					Use logarithmic scale for better visualization of IBD range
				</p>
			</div>
			
			<!-- Switch Toggle -->
			<label class="relative inline-flex items-center cursor-pointer">
				<input type="checkbox" bind:checked={logScale} class="sr-only peer">
				<div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 dark:peer-focus:ring-indigo-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-indigo-600"></div>
				<span class="ml-3 text-sm font-medium text-gray-700 dark:text-gray-300">Log₁₀</span>
			</label>
		</div>
	{/if}

	{#if (!asymmetricMode && selectedGroups.size === 0) || (asymmetricMode && (selectedXGroups.size === 0 || selectedYGroups.size === 0))}
		<div class="w-full h-[600px] md:h-[700px] lg:h-[750px] flex items-center justify-center">
			<div class="text-center">
				<div class="text-4xl mb-4">📊</div>
				<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Select Groups</h3>
				<p class="text-gray-600 dark:text-gray-400">
					{asymmetricMode ? 'Choose X and Y axis groups above to generate the heatmap' : 'Choose groups above to generate the heatmap'}
				</p>
			</div>
		</div>
	{:else}
		<div class="w-full h-[600px] md:h-[700px] lg:h-[750px] relative">
			{#if ibdLoading || plotRendering}
				<div class="absolute inset-0 flex items-center justify-center bg-white dark:bg-gray-800 z-10">
					<div class="text-center">
						<svg class="animate-spin h-8 w-8 text-indigo-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300">Computing IBD Heatmap</h3>
					</div>
				</div>
			{/if}
			<div bind:this={heatmapDiv} class="w-full h-full"></div>
		</div>
	{/if}
</div>
