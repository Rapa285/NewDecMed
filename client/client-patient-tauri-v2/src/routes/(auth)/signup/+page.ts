import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { signUpSchemaStep5 } from '$lib/schema';

export const load: PageLoad = async ({ parent, url }) => {
	const { redirect_to } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}

	const signUpForm = await superValidate(zod(signUpSchemaStep5));

	return {
		signUpForm
	};
};
