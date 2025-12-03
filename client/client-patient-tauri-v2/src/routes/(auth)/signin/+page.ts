import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { signInSchemaStep3 } from '$lib/schema';
import { zod } from 'sveltekit-superforms/adapters';

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
