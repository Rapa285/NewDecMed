import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent, params, url }) => {
	await parent();

	const patientIotaAddress = params.patientAddress;
	const accessToken = url.searchParams.get('accessToken');
	const index = url.searchParams.get('index');

	if (!accessToken || !index) {
		return error(404);
	}

	return {
		accessToken,
		patientIotaAddress,
		index: isNaN(parseInt(index)) ? 0 : parseInt(index)
	};
};
