import type { PageLoad } from './$types';
import type { CompleteProfileAdminSchema, CompleteProfilePersonnelSchema } from '$lib/types';
import { superValidate, type Infer, type SuperValidated } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { completeProfileAdminSchema, completeProfilePersonnelSchema } from '$lib/schema';
import { redirect } from '@sveltejs/kit';

type PageLoadData = {
	completeProfileAdminForm: SuperValidated<Infer<CompleteProfileAdminSchema>>;
	completeProfilePersonnelForm: SuperValidated<Infer<CompleteProfilePersonnelSchema>>;
};

export const load: PageLoad = async ({ parent, url }) => {
	const { redirect_to } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}

	const completeProfileAdminForm = await superValidate(zod(completeProfileAdminSchema));
	const completeProfilePersonnelForm = await superValidate(zod(completeProfilePersonnelSchema));

	const defaultData: PageLoadData = {
		completeProfileAdminForm,
		completeProfilePersonnelForm
	};

	return defaultData;
};
