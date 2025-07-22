<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';

	let data: any[] = [];
	let plotDiv: HTMLDivElement;
	let pcX = 0;
	let pcY = 1;
	let Plotly: any;
	let loading = true;
	
	// Visualization type selection
	let activeTab = 'pca'; // 'pca' or 'ibd'

	// Metadata fields for selection
	let availableFields = ['phs', 'country', 'region', 'sex', 'ethnicity', 'self_described', 'ibd_community'];
	let selectedFields = new Set(['ethnicity']);

	// IBD Heatmap data
	let ibdGroups: Array<{label: string, size: number}> = [];
	let selectedGroups = new Set<string>();
	let heatmapDiv: HTMLDivElement;
	let ibdLoading = false;
	let plotRendering = false;
	let heatmapData: any = null;
	let logScale = false; // Toggle for log scale visualization
	
	// IBD grouping fields
	let ibdSelectedFields = new Set(['ibd_community']); // Default to ibd_community
	let asymmetricMode = false; // Toggle for asymmetric heatmap
	let ibdXFields = new Set(['ibd_community']); // X-axis fields for asymmetric mode
	let ibdYFields = new Set(['ibd_community']); // Y-axis fields for asymmetric mode
	
	// IBD Groups for asymmetric mode
	let ibdXGroups: Array<{label: string, size: number}> = [];
	let ibdYGroups: Array<{label: string, size: number}> = [];
	let selectedXGroups = new Set<string>();
	let selectedYGroups = new Set<string>();

	// Submit Query section state
	let selectedQueryGroup = '';
	let isSubmitDisabled = true;

	// Reactive statement to update plot when loading completes
	$: if (!loading && data.length && Plotly && plotDiv) {
		setTimeout(() => updatePlot(), 100);
	}

	function toggleField(field: string) {
		if (selectedFields.has(field)) {
			selectedFields = new Set([...selectedFields].filter(f => f !== field));
		} else {
			selectedFields = new Set([field, ...selectedFields]);
		}
		updatePlot();
	}
	
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

	function updatePlot() {
		if (!data.length || !Plotly || !plotDiv) return;

		// Always order selectedFields by their order in availableFields
		const orderedFields = availableFields.filter(f => selectedFields.has(f));

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

		const traces = sortedEntries.map(([key, group]) => ({
			x: group.map(d => d.pc[pcX]),
			y: group.map(d => d.pc[pcY]),
			text: group.map(d => d.id),
			mode: 'markers',
			type: 'scatter',
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
			legend: {
				x: 1,
				y: 1,
				xanchor: 'left',
				yanchor: 'top'
			}
		};

		Plotly.newPlot(plotDiv, traces, layout, {
			responsive: true,
			displayModeBar: true,
			modeBarButtonsToRemove: ['pan2d', 'lasso2d', 'select2d', 'autoScale2d'],
			displaylogo: false
		});
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

	// Submit Query function (placeholder)
	function submitQuery() {
		// TODO: Implement submit query functionality
		console.log('Submit Query clicked for group:', selectedQueryGroup);
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

	// Load IBD groups when tab becomes active
	$: if (activeTab === 'ibd' && !asymmetricMode && ibdGroups.length === 0) {
		loadIbdGroups();
	}

	// Load asymmetric groups when tab becomes active in asymmetric mode  
	$: if (activeTab === 'ibd' && asymmetricMode && (ibdXGroups.length === 0 || ibdYGroups.length === 0)) {
		loadAsymmetricGroups();
	}

	// Update symmetric heatmap when selection or log scale changes
	$: if (activeTab === 'ibd' && !asymmetricMode && selectedGroups.size > 0 && Plotly && heatmapDiv) {
		setTimeout(() => updateHeatmap(), 100);
	}

	// Update asymmetric heatmap when selection changes
	$: if (activeTab === 'ibd' && asymmetricMode && selectedXGroups.size > 0 && selectedYGroups.size > 0 && Plotly && heatmapDiv) {
		setTimeout(() => updateAsymmetricHeatmap(), 100);
	}
	
	// Re-render heatmap when log scale toggle changes
	$: if (activeTab === 'ibd' && heatmapData && Plotly && heatmapDiv && logScale !== undefined) {
		setTimeout(async () => await renderHeatmap(), 100);
	}

	// Reactive: Generate available groups for Query dropdown
	$: availableQueryGroups = (() => {
		if (activeTab === 'pca') {
			// For PCA: extract groups from legend (same logic as updatePlot)
			if (!data.length) return [];
			
			const orderedFields = availableFields.filter(f => selectedFields.has(f));
			const grouped = new Map<string, any[]>();
			
			for (const d of data) {
				const key = orderedFields.length > 0
					? orderedFields.map(f => d[f] ?? 'Unknown').join(' | ')
					: 'All';
				if (!grouped.has(key)) grouped.set(key, []);
				grouped.get(key)!.push(d);
			}
			
			// Return groups sorted by size (descending) with group name and count
			return Array.from(grouped.entries())
				.sort((a, b) => b[1].length - a[1].length)
				.map(([key, group]) => `(${group.length}) ${key}`);
		} else if (activeTab === 'ibd') {
			// For IBD: combine selected groups from current mode and sort by size
			if (asymmetricMode) {
				// Combine X and Y groups, remove duplicates, sort by size
				const allGroups = new Set([...selectedXGroups, ...selectedYGroups]);
				const groupsArray = Array.from(allGroups);
				
				// Create a map to get group sizes for sorting
				const groupSizeMap = new Map();
				ibdXGroups.forEach(g => groupSizeMap.set(g.label, g.size));
				ibdYGroups.forEach(g => groupSizeMap.set(g.label, g.size));
				
				return groupsArray.sort((a, b) => {
					const sizeA = groupSizeMap.get(a) || 0;
					const sizeB = groupSizeMap.get(b) || 0;
					return sizeB - sizeA; // Sort by size descending
				});
			} else {
				// Use symmetric selected groups, sort by size
				const groupsArray = Array.from(selectedGroups);
				const groupSizeMap = new Map(ibdGroups.map(g => [g.label, g.size]));
				
				return groupsArray.sort((a, b) => {
					const sizeA = groupSizeMap.get(a) || 0;
					const sizeB = groupSizeMap.get(b) || 0;
					return sizeB - sizeA; // Sort by size descending
				});
			}
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
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
					<!-- PCA Controls -->
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
										on:click={() => toggleField(field)}
									>
										{field}
									</button>
								{/each}
							</div>
						</div>
					</div>
				</div>

				<!-- PCA Plot container -->
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
					<div bind:this={plotDiv} class="w-full h-[600px] md:h-[700px] lg:h-[750px]"></div>
				</div>

				<!-- Submit Query for Group Summary Data -->
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
					<div class="space-y-4">
						<h3 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Submit Query for Group Summary Data</h3>
						
						<div class="flex items-end space-x-4">
							<div class="flex-1">
								<label for="pca-group-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									Select Group:
								</label>
								<select 
									id="pca-group-select"
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

			<!-- IBD Heatmap Tab -->
			{#if activeTab === 'ibd'}
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
									{#if selectedGroups.size > 0}
										<button
											class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
											on:click={deselectAllGroups}
										>
											Deselect All
										</button>
									{/if}
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
											<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
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
													<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
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
													<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
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
											<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
										</svg>
										<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300">Computing IBD Heatmap</h3>
									</div>
								</div>
							{/if}
							<div bind:this={heatmapDiv} class="w-full h-full"></div>
						</div>
					{/if}
				</div>

				<!-- Submit Query for Group Summary Data -->
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
					<div class="space-y-4">
						<h3 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Submit Query for Group Summary Data</h3>
						
						<div class="flex items-end space-x-4">
							<div class="flex-1">
								<label for="ibd-group-select" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									Select Group:
								</label>
								<select 
									id="ibd-group-select"
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
		{/if}
	</div>
</div>
