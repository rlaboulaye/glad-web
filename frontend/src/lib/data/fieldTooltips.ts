/**
 * Centralized field tooltip definitions for metadata fields
 * Used by both hover tooltips and inline descriptions
 */

export const FIELD_TOOLTIPS: Record<string, string> = {
	phs: 'Unique identifier assigned to a phenotype study by dbGaP (some larger cohorts use cohort name instead)',
	country: 'Country where the sample was taken',
	region: 'Region where the sample was taken',
	sex: 'Reported sex of sample',
	ethnicity: 'One of ["Hispanic", "NotHispanic", "NativeAmerican", "Unknown"]',
	ethnicity_source: 'Method used to determine ethnicity ["survey_defined", "admixture_defined", "Unknown"]',
	ibd_community: 'Community assignment produced by running Infomap on total pairwise IBD between samples (numbering refers to community size in original analysis)'
};

/**
 * Get tooltip text for a metadata field
 */
export function getFieldTooltip(field: string): string {
	return FIELD_TOOLTIPS[field] || '';
}
