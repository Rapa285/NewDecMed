import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutLoad = async ({ parent, url }) => {
	const { redirect_to } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}
};
