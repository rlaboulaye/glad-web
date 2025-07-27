<script lang="ts">
	import { toast } from '$lib/toast.js';
	import MetadataFieldSelector from './MetadataFieldSelector.svelte';
	import GroupSelector from './GroupSelector.svelte';

	// Props
	export let availableFields: string[];
	export let selectedFields: Set<string>; // Shared field selections like PCA (used for X-axis in asymmetric)
	export let asymmetricMode: boolean = false;
	export let groups: Array<{label: string, size: number}> = [];
	export let selectedGroups: Set<string> = new Set(); // Used for X-axis in asymmetric
	export let yGroups: Array<{label: string, size: number}> = [];
	export let selectedYGroups: Set<string> = new Set();
	export let loading: boolean = false;
	export let Plotly: any = null;
	export let isActive: boolean = false;
	
	// Parent's yFields for asymmetric mode
	export let yFields: Set<string> = new Set();
	
	// State tracking for group reconciliation
	export let groupsReconciledForFields: Set<string> = new Set();
	export let yGroupsReconciledForFields: Set<string> = new Set();

	// Helper functions to check if groups are reconciled for current field selection
	function setsEqual<T>(a: Set<T>, b: Set<T>): boolean {
		return a.size === b.size && [...a].every(x => b.has(x));
	}
	
	function groupsAreReconciledForCurrentFields(): boolean {
		return setsEqual(groupsReconciledForFields, selectedFields);
	}
	
	function yGroupsAreReconciledForCurrentFields(): boolean {
		return setsEqual(yGroupsReconciledForFields, yFields);
	}
	
	// IBD Heatmap data
	let heatmapDiv: HTMLDivElement;
	let plotRendering = false;
	let heatmapData: any = null;
	let logScale = false; // Toggle for log scale visualization
	let maxColorValue = 1.0; // Maximum value for colorbar scale
	
	// Convert slider position to actual color value using logarithmic scale
	function sliderToColorValue(sliderPos: number): number {
		// Map slider (0-100) to logarithmic range (0.05-10)
		const minVal = 0.05;
		const maxVal = 10.0;
		const logMin = Math.log(minVal);
		const logMax = Math.log(maxVal);
		
		// Linear interpolation in log space
		const logValue = logMin + (logMax - logMin) * (sliderPos / 100);
		return Math.exp(logValue);
	}
	
	// Convert color value back to slider position  
	function colorValueToSlider(colorVal: number): number {
		const minVal = 0.05;
		const maxVal = 10.0;
		const logMin = Math.log(minVal);
		const logMax = Math.log(maxVal);
		
		const logValue = Math.log(Math.max(minVal, Math.min(maxVal, colorVal)));
		return ((logValue - logMin) / (logMax - logMin)) * 100;
	}
	
	// Initialize slider position based on default color value (1.0)
	let sliderValue = colorValueToSlider(1.0); // ~67.7 for value 1.0
	
	// Update color value when slider changes
	function handleSliderChange(event: Event) {
		const target = event.target as HTMLInputElement;
		sliderValue = parseFloat(target.value);
		maxColorValue = sliderToColorValue(sliderValue);
	}


	// Remove event handlers - parent manages all state

	// Remove group event handlers - parent manages all state

	// IBD Heatmap functions

	async function updateHeatmap() {
		if (!selectedGroups.size || !Plotly || !heatmapDiv || selectedFields.size === 0 || groups.length === 0) {
			return;
		}

		plotRendering = true;
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
			plotRendering = false;
		}
	}

	async function updateAsymmetricHeatmap() {
		// X-axis uses main selectedGroups and selectedFields, Y-axis uses selectedYGroups and yFields from parent
		if (!selectedGroups.size || !selectedYGroups.size || !Plotly || !heatmapDiv || 
			selectedFields.size === 0) return;

		// Get yFields from parent component (passed as prop)
		const parentYFields = yGroups && yGroups.length > 0 ? 
			availableFields.filter(f => yFields && yFields.has && yFields.has(f)) : 
			availableFields.filter(f => selectedFields.has(f)); // fallback to main fields

		if (parentYFields.length === 0) return;

		plotRendering = true;
		try {
			// Generate grouping parameters from selected fields
			const xOrderedFields = availableFields.filter(f => selectedFields.has(f)); // X-axis uses main fields
			const yOrderedFields = parentYFields; // Y-axis uses yFields from parent
			const xGrouping = xOrderedFields.join(',');
			const yGrouping = yOrderedFields.join(',');
			
			
			const res = await fetch('/api/ibd-matrix-asymmetric', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					row_grouping: yGrouping,      // Y Axis → rows  
					column_grouping: xGrouping,   // X Axis → columns
					selected_row_groups: [...selectedYGroups],    // Y Axis → rows
					selected_column_groups: [...selectedGroups] // X Axis → columns (main groups)
				})
			});

			if (!res.ok) {
				const errorText = await res.text();
				console.error('Asymmetric IBD matrix API error:', res.status, errorText);
				throw new Error(`Failed to compute asymmetric IBD matrix: ${res.status} ${errorText}`);
			}

			heatmapData = await res.json();
			await renderHeatmap();

		} catch (err) {
			toast.error('Failed to compute asymmetric IBD matrix');
			console.error('Error computing asymmetric IBD matrix:', err);
		} finally {
			plotRendering = false;
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
			zmin: logScale ? Math.log10(0.001) : 0, // Minimum colorbar value
			zmax: logScale ? Math.log10(maxColorValue) : maxColorValue, // Maximum colorbar value
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
	$: if (selectedGroups || selectedYGroups || asymmetricMode) {
		queryGroupsUpdateTrigger = Date.now();
	}

	// Get available groups for query dropdown (filtered by minimum size)
	export function getAvailableQueryGroups() {
		const MIN_QUERY_GROUP_SIZE = 30;
		
		if (asymmetricMode) {
			// Combine X and Y groups, remove duplicates, filter by size, sort by size
			const allGroups = new Set([...selectedGroups, ...selectedYGroups]);
			const groupsArray = Array.from(allGroups);
			
			// Create a map to get group sizes for filtering and sorting
			const groupSizeMap = new Map();
			groups.forEach(g => groupSizeMap.set(g.label, g.size)); // X-axis uses main groups
			yGroups.forEach(g => groupSizeMap.set(g.label, g.size));
			
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
			const groupSizeMap = new Map(groups.map(g => [g.label, g.size]));
			
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

	// Update symmetric heatmap when selection changes (groups managed by parent)
	// Only update when groups are reconciled for current field selection
	$: if (isActive && !asymmetricMode && selectedGroups.size > 0 && Plotly && heatmapDiv && groups.length > 0 && !loading && groupsAreReconciledForCurrentFields()) {
		setTimeout(() => updateHeatmap(), 100);
	}


	// Update asymmetric heatmap when selection or fields change (groups managed by parent)
	// Only update when BOTH X and Y groups are reconciled for their respective field selections
	$: if (isActive && asymmetricMode && selectedGroups.size > 0 && selectedYGroups.size > 0 && Plotly && heatmapDiv && groups.length > 0 && yGroups.length > 0 && !loading && groupsAreReconciledForCurrentFields() && yGroupsAreReconciledForCurrentFields()) {
		setTimeout(() => updateAsymmetricHeatmap(), 100);
	}
	
	// Re-render heatmap when scale settings change
	$: if (isActive && heatmapData && Plotly && heatmapDiv && (logScale !== undefined || maxColorValue)) {
		setTimeout(async () => await renderHeatmap(), 100);
	}

	// Clear heatmap when no metadata fields are selected
	$: if (isActive && Plotly && heatmapDiv) {
		const shouldClear = !asymmetricMode ? 
			selectedFields.size === 0 : // Symmetric: clear when no X fields
			selectedFields.size === 0 || yFields.size === 0; // Asymmetric: clear when no X or Y fields
		
		if (shouldClear) {
			Plotly.purge(heatmapDiv);
			heatmapData = null;
		}
	}
</script>

<style>
	/* Custom slider styling */
	.slider::-webkit-slider-thumb {
		appearance: none;
		height: 20px;
		width: 20px;
		border-radius: 50%;
		background: #4f46e5;
		cursor: pointer;
		border: 2px solid #ffffff;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.slider::-moz-range-thumb {
		height: 20px;
		width: 20px;
		border-radius: 50%;
		background: #4f46e5;
		cursor: pointer;
		border: 2px solid #ffffff;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.slider::-webkit-slider-track {
		height: 8px;
		border-radius: 4px;
		background: #e5e7eb;
	}

	.slider::-moz-range-track {
		height: 8px;
		border-radius: 4px;
		background: #e5e7eb;
		border: none;
	}

	:global(.dark) .slider::-webkit-slider-track {
		background: #374151;
	}

	:global(.dark) .slider::-moz-range-track {
		background: #374151;
	}
</style>

<!-- Note: MetadataFieldSelector and GroupSelector are now handled by parent -->

<!-- IBD Heatmap container -->
<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<!-- Visualization Scale Controls -->
	{#if ((!asymmetricMode && selectedGroups.size > 0) || (asymmetricMode && selectedGroups.size > 0 && selectedYGroups.size > 0))}
		<div class="mb-6 pb-4 border-b border-gray-200 dark:border-gray-600">
			<p class="text-gray-700 dark:text-gray-300 font-medium mb-4">Visualization Scale</p>
			
			<!-- Controls Row -->
			<div class="flex items-center justify-between gap-8">
				<!-- Colorbar Max Slider -->
				<div class="flex-1 flex items-center gap-4">
					<label for="max-color-slider" class="text-sm font-medium text-gray-700 dark:text-gray-300 whitespace-nowrap">
						Colorbar Max:
					</label>
					<div class="flex-1 flex items-center gap-3">
						<input 
							id="max-color-slider"
							type="range" 
							min="0" 
							max="100" 
							step="0.5" 
							value={sliderValue}
							on:input={handleSliderChange}
							class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700 slider"
						>
						<span class="text-sm text-gray-600 dark:text-gray-400 font-mono min-w-[3.5rem]">
							{#if logScale}
								{Math.log10(maxColorValue).toFixed(2)}
							{:else}
								{maxColorValue >= 1 ? maxColorValue.toFixed(1) : maxColorValue.toFixed(2)}
							{/if}
						</span>
					</div>
				</div>
				
				<!-- Log Scale Toggle -->
				<div class="flex items-center gap-3">
					<span class="text-sm font-medium text-gray-700 dark:text-gray-300">Log₁₀ Scale:</span>
					<label class="relative inline-flex items-center cursor-pointer">
						<input type="checkbox" bind:checked={logScale} class="sr-only peer">
						<div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 dark:peer-focus:ring-indigo-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-indigo-600"></div>
					</label>
				</div>
			</div>
		</div>
	{/if}

	{#if (!asymmetricMode && selectedGroups.size === 0) || (asymmetricMode && (selectedGroups.size === 0 || selectedYGroups.size === 0))}
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
			{#if loading || plotRendering}
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
