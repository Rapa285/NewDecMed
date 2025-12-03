import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ parent, url }) => {
	// const resInvokeIsAppActivated = await tryCatchAsVal(async () => {
	// 	return (await invoke('is_app_activated')) as SuccessResponse<null>;
	// });
	// if (!resInvokeIsAppActivated.success) {
	// 	return redirect(301, '/activation');
	// }

	// const resInvokeIsSignedUp = await tryCatchAsVal(async () => {
	// 	return (await invoke('is_signed_up')) as SuccessResponse<null>;
	// });
	// if (!resInvokeIsSignedUp.success) {
	// 	return redirect(301, '/signup');
	// }

	// const resInvokeIsSignedIn = await tryCatchAsVal(async () => {
	// 	return (await invoke('is_signed_in')) as SuccessResponse<null>;
	// });
	// if (!resInvokeIsSignedIn.success) {
	// 	return redirect(301, '/signin');
	// }

	const { redirect_to } = await parent();

	if (redirect_to != null && redirect_to != url.pathname) {
		return redirect(301, redirect_to);
	}
};
