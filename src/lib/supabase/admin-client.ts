import { createClient } from '@supabase/supabase-js';
import { Database } from '../types/database.types.js';
import { supabaseUrl, supabaseServiceRoleKey } from './client';


if (!supabaseUrl || !supabaseServiceRoleKey) {
    throw new Error('Missing admin credentials');
}

export const superAdmin = createClient<Database>(supabaseUrl, supabaseServiceRoleKey, {
    auth: {
        persistSession: false,
        autoRefreshToken: false
        }
    });

    /**
     * Execute a function with the admin client
     * @param callback The function to execute with the admin client
     * @returns The result of the callback function
     */
    export async function withAdminClient<T>(
    callback: (client: typeof superAdmin) => Promise<T>
    ): Promise<T> {
        try {
            return await callback(superAdmin);
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
    export async function resetUserPassword(userId: string, newPassword: string) {
        const { error } = await superAdmin.auth.admin.updateUserById(userId, {
            password: newPassword
            });
            if (error) throw error;
        }

    /**
     * Delete a user and all their data (admin only)
     * @param userId The ID of the user to delete
     */
    export async function deleteUserComplete(userId: string) {
        const { error: dbError } = await superAdmin.rpc('delete_user_data', {
            user_id_param: userId
            });
            if (dbError) throw dbError;
        
            const { error: authError } = await superAdmin.auth.admin.deleteUser(userId);
            if (authError) throw authError;
        }

    /**
     * Assign a role to a user (admin only)
     * @param userId The ID of the user
     * @param role The role to assign
     */
    export async function assignUserRole(userId: string, role: 'admin' | 'seller' | 'user') {
        const { data: existingRole, error: checkError } = await superAdmin
            .from('user_roles')
            .select('*')
            .eq('user_id', userId)
            .eq('role', role)
            .single();
        
            if (checkError && checkError.code !== 'PGRST116') throw checkError;
            if (existingRole) return;
        
            const { error } = await superAdmin.from('user_roles').insert({
            user_id: userId,
            role
            });
            if (error) throw error;
        }

    /**
     * Remove a role from a user (admin only)
     * @param userId The ID of the user
     * @param role The role to remove
     */
    export async function removeUserRole(userId: string, role: 'admin' | 'seller' | 'user'): Promise<void> {
    try {
        const { error } = await superAdmin
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