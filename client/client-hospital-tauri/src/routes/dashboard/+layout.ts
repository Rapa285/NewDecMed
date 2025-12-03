import type { Role } from '$lib/types';
import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';

type LayoutLoadData = {
	role: Role | null;
};

export const load: LayoutLoad = async ({ parent, url }) => {
	const { redirect_to, role } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}

	const defaultData: LayoutLoadData = {
		role
	};

	return defaultData;
};
