<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';

	let data: any[] = [];
	let plotDiv: HTMLDivElement;
	let pcX = 0;
	let pcY = 1;
	let Plotly: any;
	let loading = true;

	// Metadata fields for selection
	let availableFields = ['phs', 'country', 'region', 'sex', 'ethnicity', 'self_described'];
	let selectedFields = new Set(['ethnicity']);

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
				Visualize genetic diversity using Principal Component Analysis (PCA)
			</p>
		</div>

		{#if loading}
			<div class="flex justify-center items-center h-64">
				<div class="text-center">
					<svg class="animate-spin h-8 w-8 text-indigo-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					<p class="text-gray-600 dark:text-gray-400">Loading PCA data...</p>
				</div>
			</div>
		{:else}
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
				<!-- Controls -->
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

			<!-- Plot container -->
			<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
				<div bind:this={plotDiv} class="w-full h-[600px] md:h-[700px] lg:h-[750px]"></div>
			</div>
		{/if}
	</div>
</div>
