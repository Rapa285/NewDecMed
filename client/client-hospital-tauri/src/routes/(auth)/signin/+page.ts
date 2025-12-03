import { superValidate } from 'sveltekit-superforms';
import type { PageLoad } from './$types';
import { zod } from 'sveltekit-superforms/adapters';
import { signInSchemaStep3 } from '$lib/schema';
import { redirect } from '@sveltejs/kit';

export const load: PageLoad = async ({ parent, url }) => {
	const { redirect_to } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}

	const signInForm = await superValidate(zod(signInSchemaStep3));

	return {
		signInForm
	};
};
