import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { createMedicalRecordSchema } from '$lib/schema';

export const load: PageLoad = async ({ parent, params, url }) => {
	await parent();

	const patientIotaAddress = params.patientAddress;
	const accessToken = url.searchParams.get('accessToken');
	const patientPrePublicKey = url.searchParams.get('patientPrePublicKey');

	if (!accessToken || !patientPrePublicKey) {
		return error(404);
	}

	const createMedicalRecordForm = await superValidate(zod(createMedicalRecordSchema));

	return {
		accessToken,
		createMedicalRecordForm,
		patientIotaAddress,
		patientPrePublicKey
	};
};
