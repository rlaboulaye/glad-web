<script lang="ts">
	import { toast } from '$lib/toast.js';

	// Props
	export let availableFields: string[];
	export let Plotly: any = null;
	export let isActive: boolean = false;

	// IBD grouping fields
	let ibdSelectedFields = new Set(['ibd_community']); // Default to ibd_community
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

	function toggleIbdField(field: string) {
		if (ibdSelectedFields.has(field)) {
			ibdSelectedFields = new Set([...ibdSelectedFields].filter(f => f !== field));
		} else {
			ibdSelectedFields = new Set([field, ...ibdSelectedFields]);
		}
		// Reset groups when grouping changes
		ibdGroups = [];
		selectedGroups = new Set();
		if (ibdSelectedFields.size > 0) {
			loadIbdGroups();
		}
	}

	function toggleAsymmetricMode() {
		asymmetricMode = !asymmetricMode;
		
		if (asymmetricMode) {
			// Copy current symmetric fields to both X and Y
			ibdXFields = new Set(ibdSelectedFields);
			ibdYFields = new Set(ibdSelectedFields);
			// Reset groups
			ibdXGroups = [];
			ibdYGroups = [];
			selectedXGroups = new Set();
			selectedYGroups = new Set();
			loadAsymmetricGroups();
		} else {
			// Copy current X axis fields to symmetric mode
			ibdSelectedFields = new Set(ibdXFields);
			// Reset groups
			ibdGroups = [];
			selectedGroups = new Set();
			if (ibdSelectedFields.size > 0) {
				loadIbdGroups();
			}
		}
	}

	function toggleIbdXField(field: string) {
		if (ibdXFields.has(field)) {
			ibdXFields = new Set([...ibdXFields].filter(f => f !== field));
		} else {
			ibdXFields = new Set([field, ...ibdXFields]);
		}
		loadAsymmetricGroups();
	}

	function toggleIbdYField(field: string) {
		if (ibdYFields.has(field)) {
			ibdYFields = new Set([...ibdYFields].filter(f => f !== field));
		} else {
			ibdYFields = new Set([field, ...ibdYFields]);
		}
		loadAsymmetricGroups();
	}

	// IBD Heatmap functions
	function toggleGroup(groupLabel: string) {
		if (selectedGroups.has(groupLabel)) {
			selectedGroups = new Set([...selectedGroups].filter(g => g !== groupLabel));
		} else {
			selectedGroups = new Set([groupLabel, ...selectedGroups]);
		}
		updateHeatmap();
	}

	function deselectAllGroups() {
		selectedGroups = new Set();
	}

	function toggleXGroup(groupLabel: string) {
		if (selectedXGroups.has(groupLabel)) {
			selectedXGroups = new Set([...selectedXGroups].filter(g => g !== groupLabel));
		} else {
			selectedXGroups = new Set([groupLabel, ...selectedXGroups]);
		}
		updateAsymmetricHeatmap();
	}

	function toggleYGroup(groupLabel: string) {
		if (selectedYGroups.has(groupLabel)) {
			selectedYGroups = new Set([...selectedYGroups].filter(g => g !== groupLabel));
		} else {
			selectedYGroups = new Set([groupLabel, ...selectedYGroups]);
		}
		updateAsymmetricHeatmap();
	}

	function deselectAllXGroups() {
		selectedXGroups = new Set();
	}

	function deselectAllYGroups() {
		selectedYGroups = new Set();
	}

	async function loadIbdGroups() {
		// Don't load if no fields are selected
		if (ibdSelectedFields.size === 0) return;
		
		try {
			// Generate grouping parameter from selected fields
			const orderedFields = availableFields.filter(f => ibdSelectedFields.has(f));
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
		if (!selectedGroups.size || !Plotly || !heatmapDiv || ibdSelectedFields.size === 0) return;

		ibdLoading = true;
		try {
			// Generate grouping parameter from selected fields
			const orderedFields = availableFields.filter(f => ibdSelectedFields.has(f));
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

	// Load IBD groups when tab becomes active
	$: if (isActive && !asymmetricMode && ibdGroups.length === 0) {
		loadIbdGroups();
	}

	// Load asymmetric groups when tab becomes active in asymmetric mode  
	$: if (isActive && asymmetricMode && (ibdXGroups.length === 0 || ibdYGroups.length === 0)) {
		loadAsymmetricGroups();
	}

	// Update symmetric heatmap when selection or log scale changes
	$: if (isActive && !asymmetricMode && selectedGroups.size > 0 && Plotly && heatmapDiv) {
		setTimeout(() => updateHeatmap(), 100);
	}

	// Update asymmetric heatmap when selection changes
	$: if (isActive && asymmetricMode && selectedXGroups.size > 0 && selectedYGroups.size > 0 && Plotly && heatmapDiv) {
		setTimeout(() => updateAsymmetricHeatmap(), 100);
	}
	
	// Re-render heatmap when log scale toggle changes
	$: if (isActive && heatmapData && Plotly && heatmapDiv && logScale !== undefined) {
		setTimeout(async () => await renderHeatmap(), 100);
	}
</script>

<!-- IBD Grouping Fields Selector -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="space-y-4">
		<div class="flex items-center justify-between">
			<p class="text-gray-700 dark:text-gray-300 font-semibold">Group IBD Heatmap by Metadata Fields:</p>
			<button
				class="px-4 py-2 text-sm font-medium rounded-md border transition-colors duration-200 bg-gray-100 dark:bg-gray-600 border-gray-300 dark:border-gray-500 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-500"
				on:click={toggleAsymmetricMode}
			>
				{asymmetricMode ? '🔄 Switch to Symmetric' : '🔄 Switch to Asymmetric'}
			</button>
		</div>
		
		{#if !asymmetricMode}
			<!-- Symmetric mode field selector -->
			<div class="flex flex-wrap gap-2">
				{#each availableFields as field}
					<button
						class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
							{ibdSelectedFields.has(field)
								? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
								: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
						title={getFieldTooltip(field)}
						on:click={() => toggleIbdField(field)}
					>
						{field}
					</button>
				{/each}
			</div>
		{:else}
			<!-- Asymmetric mode X/Y axis selectors -->
			<div class="space-y-4">
				<!-- X Axis -->
				<div>
					<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">X Axis:</p>
					<div class="flex flex-wrap gap-2">
						{#each availableFields as field}
							<button
								class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
									{ibdXFields.has(field)
										? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								title={getFieldTooltip(field)}
								on:click={() => toggleIbdXField(field)}
							>
								{field}
							</button>
						{/each}
					</div>
				</div>
				
				<!-- Y Axis -->
				<div>
					<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">Y Axis:</p>
					<div class="flex flex-wrap gap-2">
						{#each availableFields as field}
							<button
								class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
									{ibdYFields.has(field)
										? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								title={getFieldTooltip(field)}
								on:click={() => toggleIbdYField(field)}
							>
								{field}
							</button>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<!-- IBD Controls -->
	<div class="space-y-6">
		{#if (!asymmetricMode && ibdSelectedFields.size === 0) || (asymmetricMode && (ibdXFields.size === 0 || ibdYFields.size === 0))}
			<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
				<div class="text-center">
					<div class="text-3xl mb-2">🏷️</div>
					<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Select Metadata Fields</h3>
					<p class="text-gray-600 dark:text-gray-400">
						Choose metadata fields above to group the data
					</p>
				</div>
			</div>
		{:else if !asymmetricMode}
			<!-- Symmetric mode group selection -->
			<div>
				<div class="flex items-center justify-between mb-4">
					<div>
						<p class="text-gray-700 dark:text-gray-300 font-semibold">Group Selection:</p>
						<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
							Choose groups to visualize Identity-by-Descent patterns ({selectedGroups.size} selected)
						</p>
					</div>
				<div class="flex items-center space-x-3">
					<button
						class="px-3 py-1.5 text-sm font-medium transition-colors duration-200 rounded-md
							{selectedGroups.size > 0 
								? 'text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 hover:bg-gray-200 dark:hover:bg-gray-500' 
								: 'text-gray-400 dark:text-gray-500 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 cursor-not-allowed'}"
						on:click={deselectAllGroups}
						disabled={selectedGroups.size === 0}
					>
						Deselect All
					</button>
				</div>
			</div>

			{#if ibdGroups.length > 0}
				<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
					<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
						{#each ibdGroups as group}
							<button
								class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
									{selectedGroups.has(group.label)
										? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								on:click={() => toggleGroup(group.label)}
								disabled={ibdLoading}
							>
								<div class="font-medium">{group.label}</div>
								<div class="text-xs opacity-75">{group.size} individuals</div>
							</button>
						{/each}
					</div>
				</div>
			{:else}
				<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
					<div class="text-center">
						<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 718-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						<p class="text-sm text-gray-600 dark:text-gray-400">Loading groups...</p>
					</div>
				</div>
			{/if}
		</div>
		{:else}
			<!-- Asymmetric mode group selection -->
			<div class="space-y-6">
				<!-- X Axis Groups -->
				<div>
					<div class="flex items-center justify-between mb-4">
						<div>
							<p class="text-gray-700 dark:text-gray-300 font-semibold">X Axis Groups:</p>
							<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
								Choose X-axis groups ({selectedXGroups.size} selected)
							</p>
						</div>
						<div class="flex items-center space-x-3">
							{#if selectedXGroups.size > 0}
								<button
									class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
									on:click={deselectAllXGroups}
								>
									Deselect All
								</button>
							{/if}
						</div>
					</div>

					{#if ibdXGroups.length > 0}
						<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
							<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
								{#each ibdXGroups as group}
									<button
										class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
											{selectedXGroups.has(group.label)
												? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
												: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
										on:click={() => toggleXGroup(group.label)}
										disabled={ibdLoading}
									>
										<div class="font-medium">{group.label}</div>
										<div class="text-xs opacity-75">{group.size} individuals</div>
									</button>
								{/each}
							</div>
						</div>
					{:else}
						<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
							<div class="text-center">
								<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 718-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								<p class="text-sm text-gray-600 dark:text-gray-400">Loading X groups...</p>
							</div>
						</div>
					{/if}
				</div>

				<!-- Y Axis Groups -->
				<div>
					<div class="flex items-center justify-between mb-4">
						<div>
							<p class="text-gray-700 dark:text-gray-300 font-semibold">Y Axis Groups:</p>
							<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
								Choose Y-axis groups ({selectedYGroups.size} selected)
							</p>
						</div>
						<div class="flex items-center space-x-3">
							{#if selectedYGroups.size > 0}
								<button
									class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
									on:click={deselectAllYGroups}
								>
									Deselect All
								</button>
							{/if}
						</div>
					</div>

					{#if ibdYGroups.length > 0}
						<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
							<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
								{#each ibdYGroups as group}
									<button
										class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
											{selectedYGroups.has(group.label)
												? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
												: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
										on:click={() => toggleYGroup(group.label)}
										disabled={ibdLoading}
									>
										<div class="font-medium">{group.label}</div>
										<div class="text-xs opacity-75">{group.size} individuals</div>
									</button>
								{/each}
							</div>
						</div>
					{:else}
						<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
							<div class="text-center">
								<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								<p class="text-sm text-gray-600 dark:text-gray-400">Loading Y groups...</p>
							</div>
						</div>
					{/if}
				</div>
			</div>
		{/if}
		
		<!-- Log Scale Toggle (moved below community selection) -->
		<div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-600">
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
	</div>
</div>

<!-- IBD Heatmap container -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
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