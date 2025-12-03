import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { addPersonnelSchemaStep2 } from '$lib/schema';

export const load: PageLoad = async ({ parent }) => {
	const { role } = await parent();

	switch (role) {
		case 'Admin': {
			const addPersonnelForm = await superValidate(zod(addPersonnelSchemaStep2));

			return {
				role,
				addPersonnelForm
			};
		}
		case 'AdministrativePersonnel': {
			break;
		}
		case 'MedicalPersonnel': {
			break;
		}
	}

	return {
		role
	};
};
