<script>
	import { user } from '$lib/auth.js';
</script>

<svelte:head>
	<title>GLAD - Genetics of Latin American Diversity</title>
</svelte:head>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center px-4 sm:px-6 lg:px-8 relative">
	<div class="max-w-4xl mx-auto text-center">
		<div class="mb-8 flex justify-center">
			<div class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-bold text-gray-800 dark:text-gray-100 leading-tight text-left">
				<div class="mb-2">
					<span class="text-green-600 dark:text-green-400">G</span><span class="ml-2">enetics of</span>
				</div>
				<div class="mb-2">
					<span class="text-green-600 dark:text-green-400">L</span><span class="ml-2">atin</span>
				</div>
				<div class="mb-2">
					<span class="text-green-600 dark:text-green-400">A</span><span class="ml-2">merican</span>
				</div>
				<div class="mb-6">
					<span class="text-green-600 dark:text-green-400">D</span><span class="ml-2">iversity</span>
				</div>
			</div>
		</div>
		
		<div class="max-w-3xl mx-auto">
			<p class="text-lg sm:text-xl md:text-2xl text-gray-600 dark:text-gray-300 leading-relaxed">
				Resources and tools to address the complexities of association and evolution in understudied and recently admixed populations.
			</p>
		</div>
		
		<div class="mt-12 flex flex-col sm:flex-row gap-4 justify-center">
			<a
				href="/explore"
				class="inline-flex items-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 transition-colors duration-200"
			>
				<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
				</svg>
				Explore Cohorts
			</a>
			<a
				href={$user ? "/find" : "/login"}
				class="inline-flex items-center px-8 py-3 border border-transparent text-base font-medium rounded-md text-green-600 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 dark:bg-gray-800 dark:text-green-400 dark:hover:bg-gray-700 transition-colors duration-200"
			>
				<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
				</svg>
				Find Controls
			</a>
		</div>
	</div>
	
	<!-- Lab Logo -->
	<div class="absolute bottom-16 left-8 hidden md:block">
		<img 
			src="/lab-logo.png" 
			alt="Evolution Genetics Group" 
			class="h-48 w-auto opacity-80 hover:opacity-100 transition-opacity duration-200"
		/>
	</div>
	
	<!-- Institute Logo and Text -->
	<div class="absolute bottom-20 right-8 hidden md:flex items-center space-x-4">
		<img 
			src="/umb_som.svg" 
			alt="University of Maryland School of Medicine" 
			class="h-36 w-auto opacity-80 hover:opacity-100 transition-opacity duration-200"
		/>
		<div class="text-right text-gray-600 dark:text-gray-400" style="font-family: TrajanProRegular, serif;">
			<div class="text-sm font-medium">UNIVERSITY OF MARYLAND</div>
			<div class="text-sm font-medium">SCHOOL OF MEDICINE</div>
			<div class="text-sm font-medium">INSTITUTE FOR GENOME SCIENCES</div>
		</div>
	</div>
</div>
