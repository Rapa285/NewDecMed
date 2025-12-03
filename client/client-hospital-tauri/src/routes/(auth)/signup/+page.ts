import { superValidate } from 'sveltekit-superforms';
import type { PageLoad } from './$types';
import { zod } from 'sveltekit-superforms/adapters';
import { signUpSchemaStep4 } from '$lib/schema';
import { redirect } from '@sveltejs/kit';

export const load: PageLoad = async ({ parent, url }) => {
	const { redirect_to } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}

	const signUpForm = await superValidate(zod(signUpSchemaStep4));

	return {
		signUpForm
	};
};
