import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { enterPinSchema, hospitalQrSchema } from '$lib/schema';

export const load: PageLoad = async ({ parent }) => {
	await parent();

	const hospitalQrForm = await superValidate(zod(hospitalQrSchema));
	const enterPinForm = await superValidate(zod(enterPinSchema));

	return {
		hospitalQrForm,
		enterPinForm
	};
};
