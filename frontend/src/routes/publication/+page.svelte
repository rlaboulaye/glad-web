<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '$lib/toast.js';

	// Publication metadata
	const publication = {
		title: "Genetics of Latin American Diversity Project: Insights into population genetics and association studies in admixed groups in the Americas",
		authors: [
			"Victor Borda", "Douglas P Loesch", "Bing Guo", "Roland Laboulaye", "Diego Veliz-Otani", 
			"Jennifer N French", "Thiago Peixoto Leal", "Stephanie M Gogarten", "Sunday Ikpe", 
			"Mateus H Gouveia", "...", "Ryan D. Hernandez", "Eduardo Tarazona-Santos", "Timothy D. O'Connor"
		],
		journal: "Cell Genomics",
		volume: "4",
		number: "11", 
		year: "2024",
		publisher: "Elsevier",
		doi: "10.1016/j.xgen.2024.100692",
		semanticScholarId: "38829ba284b340c6250906a6a951a32efadd9740",
		summary: `Latin Americans are underrepresented in genetic studies, increasing disparities in personalized genomic medicine. Despite available genetic data from thousands of Latin Americans, accessing and navigating the bureaucratic hurdles for consent or access remains challenging. To address this, we introduce the Genetics of Latin American Diversity (GLAD) Project, compiling genome-wide information from 53,738 Latin Americans across 39 studies representing 46 geographical regions. Through GLAD, we identified heterogeneous ancestry composition and recent gene flow across the Americas. Additionally, we developed GLAD-match, a simulated annealing-based algorithm, to match the genetic background of external samples to our database, sharing summary statistics (i.e., allele and haplotype frequencies) without transferring individual-level genotypes. Finally, we demonstrate the potential of GLAD as a critical resource for evaluating statistical genetic software in the presence of admixture. By providing this resource, we promote genomic research in Latin Americans and contribute to the promises of personalized medicine to more people.`
	};

	const bibtex = `@article{borda2024genetics,
  title={Genetics of Latin American Diversity Project: Insights into population genetics and association studies in admixed groups in the Americas},
  author={Borda, Victor and Loesch, Douglas P and Guo, Bing and Laboulaye, Roland and Veliz-Otani, Diego and French, Jennifer N and Leal, Thiago Peixoto and Gogarten, Stephanie M and Ikpe, Sunday and Gouveia, Mateus H and others},
  journal={Cell genomics},
  volume={4},
  number={11},
  year={2024},
  publisher={Elsevier}
}`;

	// Citations data
	let citations: any[] = [];
	let citationCount = 0;
	let loadingCitations = true;
	let citationError = false;

	// Load citations from our API (which proxies Semantic Scholar)
	async function loadCitations() {
		try {
			loadingCitations = true;
			const response = await fetch(`/api/citations?paper_id=${publication.semanticScholarId}`);
			
			if (!response.ok) {
				throw new Error('Failed to load citations');
			}

			const data = await response.json();
			citations = data.data || [];
			citationCount = data.total || citations.length;
		} catch (error) {
			console.error('Error loading citations:', error);
			citationError = true;
			toast.error('Failed to load citation data');
		} finally {
			loadingCitations = false;
		}
	}

	// Copy text to clipboard
	async function copyToClipboard(text: string, type: string) {
		try {
			await navigator.clipboard.writeText(text);
			toast.success(`${type} copied to clipboard!`);
		} catch (error) {
			console.error('Failed to copy:', error);
			toast.error('Failed to copy to clipboard');
		}
	}

	onMount(() => {
		loadCitations();
	});
</script>

<svelte:head>
	<title>Publication - GLAD</title>
	<meta name="description" content="Genetics of Latin American Diversity Project publication details and citations" />
</svelte:head>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-4xl mx-auto">
		<!-- Header -->
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-gray-800 dark:text-gray-100">Publication</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				Research behind the GLAD project
			</p>
		</div>

		<!-- Paper Details -->
		<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-8 mb-8">
			<h2 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-4">
				{publication.title}
			</h2>
			
			<div class="mb-6 text-gray-700 dark:text-gray-300">
				<p class="mb-2">
					<span class="font-semibold">Authors:</span> 
					{publication.authors.join(', ')}
				</p>
				<p class="mb-2">
					<span class="font-semibold">Journal:</span> 
					{publication.journal}, Volume {publication.volume}, Number {publication.number} ({publication.year})
				</p>
				<p class="mb-2">
					<span class="font-semibold">Publisher:</span> {publication.publisher}
				</p>
				<p class="mb-4">
					<span class="font-semibold">DOI:</span> 
					<a 
						href="https://doi.org/{publication.doi}" 
						target="_blank" 
						rel="noopener noreferrer"
						class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-700 dark:hover:text-indigo-300 underline"
					>
						{publication.doi}
					</a>
				</p>
			</div>

			<div class="mb-6">
				<h3 class="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-3">Summary</h3>
				<p class="text-gray-700 dark:text-gray-300 leading-relaxed">
					{publication.summary}
				</p>
			</div>

			<div class="flex flex-wrap gap-3">
				<a
					href="https://doi.org/{publication.doi}"
					target="_blank"
					rel="noopener noreferrer"
					class="inline-flex items-center px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700 transition-colors duration-200"
				>
					Read Paper
					<svg class="ml-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
			</div>
		</div>

		<!-- Citation Section -->
		<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-8 mb-8">
			<h3 class="text-xl font-semibold text-gray-800 dark:text-gray-100 mb-4">How to Cite</h3>
			
			<div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 relative">
				<button
					on:click={() => copyToClipboard(bibtex, 'BibTeX citation')}
					class="absolute top-2 right-2 p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors duration-200"
					title="Copy BibTeX to clipboard"
				>
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
					</svg>
				</button>
				<pre class="text-sm text-gray-800 dark:text-gray-200 overflow-x-auto pr-12 font-mono">{bibtex}</pre>
			</div>
		</div>

		<!-- Citations -->
		<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-8">
			<h3 class="text-xl font-semibold text-gray-800 dark:text-gray-100 mb-4">
				Citations 
				{#if citationCount > 0}
					<span class="text-gray-500 dark:text-gray-400 font-normal">({citationCount})</span>
				{/if}
			</h3>

			{#if loadingCitations}
				<div class="flex items-center justify-center py-8">
					<svg class="animate-spin h-6 w-6 text-indigo-600 mr-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					<span class="text-gray-600 dark:text-gray-400">Loading citations...</span>
				</div>
			{:else if citationError}
				<div class="text-center py-8">
					<div class="text-gray-500 dark:text-gray-400 mb-4">
						<svg class="mx-auto h-12 w-12 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
						<p>Unable to load citation data</p>
					</div>
					<button
						on:click={loadCitations}
						class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-700 dark:hover:text-indigo-300 underline"
					>
						Try again
					</button>
				</div>
			{:else if citations.length === 0}
				<div class="text-center py-8">
					<div class="text-gray-500 dark:text-gray-400">
						<svg class="mx-auto h-12 w-12 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
						</svg>
						<p>No citations found yet</p>
						<p class="text-sm mt-1">This is a recent publication - citations may appear over time</p>
					</div>
				</div>
			{:else}
				<div class="space-y-4">
					{#each citations as citation}
						<div class="border border-gray-200 dark:border-gray-600 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200">
							<h4 class="font-medium text-gray-800 dark:text-gray-100 mb-2">
								{citation.title}
							</h4>
							
							{#if citation.authors && citation.authors.length > 0}
								<p class="text-sm text-gray-600 dark:text-gray-400 mb-1">
									{citation.authors.slice(0, 3).join(', ')}
									{#if citation.authors.length > 3}
										, et al.
									{/if}
								</p>
							{/if}
							
							<div class="flex items-center justify-between">
								<div class="text-sm text-gray-500 dark:text-gray-400">
									{#if citation.venue}
										{citation.venue}
									{/if}
									{#if citation.year}
										{citation.venue ? ' • ' : ''}{citation.year}
									{/if}
								</div>
								
								{#if citation.open_access_pdf}
									<a
										href={citation.open_access_pdf}
										target="_blank"
										rel="noopener noreferrer" 
										class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-700 dark:hover:text-indigo-300 text-sm underline"
									>
										View PDF
									</a>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}

			{#if citations.length > 0}
				<div class="mt-6 pt-6 border-t border-gray-200 dark:border-gray-600">
					<p class="text-sm text-gray-500 dark:text-gray-400 text-center">
						Citation data provided by <a href="https://www.semanticscholar.org/" target="_blank" rel="noopener noreferrer" class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-700 dark:hover:text-indigo-300 underline">Semantic Scholar</a>
					</p>
				</div>
			{/if}
		</div>
	</div>
</div>
