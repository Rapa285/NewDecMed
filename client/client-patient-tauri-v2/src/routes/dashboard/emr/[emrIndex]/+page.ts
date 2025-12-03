import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent, params }) => {
	await parent();

	const emrIndex = parseInt(params.emrIndex);

	return { emrIndex };
};
