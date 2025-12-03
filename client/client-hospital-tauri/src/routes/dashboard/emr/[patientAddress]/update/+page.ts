import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { updateMedicalRecordSchema } from '$lib/schema';

export const load: PageLoad = async ({ parent, params, url }) => {
	await parent();

	const patientIotaAddress = params.patientAddress;
	const accessToken = url.searchParams.get('accessToken');
	const patientPrePublicKey = url.searchParams.get('patientPrePublicKey');
	const medicalMetadataIndex = url.searchParams.get('medicalMetadataIndex');

	if (!accessToken || !patientPrePublicKey) {
		return error(404);
	}

	const updateMedicalRecordForm = await superValidate(zod(updateMedicalRecordSchema));

	return {
		accessToken,
		medicalMetadataIndex: isNaN(parseInt(medicalMetadataIndex || ''))
			? null
			: parseInt(medicalMetadataIndex!),
		patientIotaAddress,
		patientPrePublicKey,
		updateMedicalRecordForm
	};
};
