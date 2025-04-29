import { adminClient } from '../../lib/supabase/client';

/**
 * Get the admin Supabase client
 * This client should only be used in trusted server environments, such as:
 * - Serverless functions
 * - Server-side API routes
 * - Backend services with proper authentication
 * 
 * NEVER use this client in client-side code or expose it to the browser.
 */
export function getAdminClient() {
    return adminClient;
    }

    /**
     * Execute a function with the admin client
     * @param callback The function to execute with the admin client
     * @returns The result of the callback function
     */
    export async function withAdminClient<T>(
    callback: (client: typeof adminClient) => Promise<T>
    ): Promise<T> {
    try {
        return await callback(adminClient);
    } catch (error) {
        console.error('Error in admin client operation:', error);
        throw error;
    }
    }

    /**
     * Reset a user's password (admin only)
     * @param userId The ID of the user
     * @param newPassword The new password
     */
    export async function resetUserPassword(userId: string, newPassword: string): Promise<void> {
        try {
            const { error } = await adminClient.auth.admin.updateUserById(userId, {
            password: newPassword,
            });

            if (error) {
            throw error;
            }
        } catch (error) {
            console.error('Error resetting user password:', error);
            throw error;
        }
    }

    /**
     * Delete a user and all their data (admin only)
     * @param userId The ID of the user to delete
     */
    export async function deleteUserComplete(userId: string): Promise<void> {
    try {
        // Begin a transaction to delete user data
        const { error: dbError } = await adminClient.rpc('delete_user_data', {
        user_id_param: userId,
        });

        if (dbError) {
        throw dbError;
        }

        // Delete the user from auth
        const { error: authError } = await adminClient.auth.admin.deleteUser(userId);

        if (authError) {
        throw authError;
        }
    } catch (error) {
        console.error('Error deleting user:', error);
        throw error;
    }
    }

    /**
     * Assign a role to a user (admin only)
     * @param userId The ID of the user
     * @param role The role to assign
     */
    export async function assignUserRole(userId: string, role: 'admin' | 'seller' | 'user'): Promise<void> {
    try {
        // Check if the role already exists
        const { data: existingRole, error: checkError } = await adminClient
        .from('user_roles')
        .select('*')
        .eq('user_id', userId)
        .eq('role', role)
        .single();

        if (checkError && checkError.code !== 'PGRST116') {
        // Error other than "no rows returned"
        throw checkError;
        }

        if (existingRole) {
        // Role already assigned
        return;
        }

        // Assign the new role
        const { error } = await adminClient.from('user_roles').insert({
        user_id: userId,
        role,
        });

        if (error) {
        throw error;
        }
    } catch (error) {
        console.error('Error assigning user role:', error);
        throw error;
    }
    }

    /**
     * Remove a role from a user (admin only)
     * @param userId The ID of the user
     * @param role The role to remove
     */
    export async function removeUserRole(userId: string, role: 'admin' | 'seller' | 'user'): Promise<void> {
    try {
        const { error } = await adminClient
        .from('user_roles')
        .delete()
        .eq('user_id', userId)
        .eq('role', role);

        if (error) {
        throw error;
        }
    } catch (error) {
        console.error('Error removing user role:', error);
        throw error;
    }
}