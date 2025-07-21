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
	let heatmapData: any = null;
	let logScale = false; // Toggle for log scale visualization

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

	async function loadIbdGroups() {
		try {
			const res = await fetch('/api/ibd-groups?grouping=ibd_community&min_size=30');
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

	async function updateHeatmap() {
		if (!selectedGroups.size || !Plotly || !heatmapDiv) return;

		ibdLoading = true;
		try {
			const res = await fetch('/api/ibd-matrix', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					grouping: 'ibd_community',
					selected_groups: [...selectedGroups]
				})
			});

			if (!res.ok) {
				throw new Error('Failed to compute IBD matrix');
			}

			heatmapData = await res.json();
			renderHeatmap();

		} catch (err) {
			toast.error('Failed to compute IBD matrix');
			console.error('Error computing IBD matrix:', err);
		} finally {
			ibdLoading = false;
		}
	}

	function renderHeatmap() {
		if (!heatmapData || !Plotly || !heatmapDiv) return;

		const { matrix, group_labels, group_sizes } = heatmapData;

		// Create sorted indices for y-axis (descending by size)
		const sortedIndices = group_labels
			.map((label, i) => ({ index: i, size: group_sizes[i], label }))
			.sort((a, b) => b.size - a.size)
			.map(item => item.index);

		// Conditionally apply log transformation based on toggle
		const processedMatrix = logScale 
			? matrix.map(row => row.map(val => val > 0 ? Math.log10(val + 0.001) : Math.log10(0.001)))
			: matrix;

		// Reorder matrix rows and y-labels by descending size
		const reorderedMatrix = sortedIndices.map(i => processedMatrix[i]).reverse(); // Reverse matrix to match labels
		const yLabels = sortedIndices.map(i => `${group_labels[i]} (${group_sizes[i]})`).reverse(); // Reverse to put largest at top
		const xLabels = group_labels.map((label, i) => `${label} (${group_sizes[i]})`);

		const trace = {
			z: reorderedMatrix,
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
				text: `IBD Heatmap: ${group_labels.length} Communities`,
				font: { size: 20, color: '#1f2937' }
			},
			autosize: true,
			xaxis: {
				title: 'IBD Community (Size)',
				side: 'bottom',
				tickangle: -45,
				automargin: true
			},
			yaxis: {
				title: 'IBD Community (Size)',
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

		Plotly.newPlot(heatmapDiv, [trace], layout, {
			responsive: true,
			displayModeBar: true,
			displaylogo: false
		});
	}

	// Load IBD groups when tab becomes active
	$: if (activeTab === 'ibd' && ibdGroups.length === 0) {
		loadIbdGroups();
	}

	// Update heatmap when selection or log scale changes
	$: if (activeTab === 'ibd' && selectedGroups.size > 0 && Plotly && heatmapDiv) {
		setTimeout(() => updateHeatmap(), 100);
	}
	
	// Re-render heatmap when log scale toggle changes
	$: if (activeTab === 'ibd' && heatmapData && Plotly && heatmapDiv && logScale !== undefined) {
		setTimeout(() => renderHeatmap(), 100);
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
							<p class="text-gray-700 dark:text-gray-300 mb-3 font-semibold">Color and filter by metadata fields:</p>
							<div class="flex flex-wrap gap-2">
								{#each availableFields as field}
									<button
										class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
											{selectedFields.has(field)
												? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
												: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
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
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
					<div bind:this={plotDiv} class="w-full h-[600px] md:h-[700px] lg:h-[750px]"></div>
				</div>
			{/if}

			<!-- IBD Heatmap Tab -->
			{#if activeTab === 'ibd'}
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
					<!-- IBD Controls -->
					<div class="space-y-6">
						<div>
							<div class="flex items-center justify-between mb-4">
								<div>
									<p class="text-gray-700 dark:text-gray-300 font-semibold">IBD Community Selection:</p>
									<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
										Choose communities to visualize Identity-by-Descent patterns ({selectedGroups.size} selected)
									</p>
								</div>
								<div class="flex items-center space-x-3">
									{#if ibdLoading}
										<div class="flex items-center space-x-2">
											<svg class="animate-spin h-5 w-5 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
												<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
												<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
											</svg>
											<span class="text-sm text-gray-600 dark:text-gray-400">Computing...</span>
										</div>
									{:else if selectedGroups.size > 0}
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
										<p class="text-sm text-gray-600 dark:text-gray-400">Loading communities...</p>
									</div>
								</div>
							{/if}
						</div>
						
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
				<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
					{#if selectedGroups.size === 0}
						<div class="w-full h-[600px] md:h-[700px] lg:h-[750px] flex items-center justify-center">
							<div class="text-center">
								<div class="text-4xl mb-4">📊</div>
								<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Select Communities</h3>
								<p class="text-gray-600 dark:text-gray-400">
									Choose IBD communities above to generate the heatmap
								</p>
							</div>
						</div>
					{:else}
						<div bind:this={heatmapDiv} class="w-full h-[600px] md:h-[700px] lg:h-[750px]"></div>
					{/if}
				</div>
			{/if}
		{/if}
	</div>
</div>
