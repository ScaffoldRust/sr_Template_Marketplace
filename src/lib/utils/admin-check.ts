import { SupabaseClient } from '@supabase/supabase-js';

/**
 * Check if a user is an admin
 * @param supabase The Supabase client
 * @param userId The ID of the user to check
 * @returns Boolean indicating if the user is an admin
 */
export async function isAdmin(supabase: SupabaseClient, userId: string): Promise<boolean> {
    try {
        const { data, error } = await supabase
        .from('user_roles')
        .select('role')
        .eq('user_id', userId)
        .eq('role', 'admin')
        .single();

        if (error) {
        console.error('Error checking admin status:', error);
        return false;
        }

        return !!data;
    } catch (error) {
        console.error('Exception in isAdmin check:', error);
        return false;
    }
    }

    /**
     * Check if a user is a seller or admin
     * @param supabase The Supabase client
     * @param userId The ID of the user to check
     * @returns Boolean indicating if the user is a seller or admin
     */
    export async function isSellerOrAdmin(supabase: SupabaseClient, userId: string): Promise<boolean> {
    try {
        const { data, error } = await supabase
        .from('user_roles')
        .select('role')
        .eq('user_id', userId)
        .in('role', ['admin', 'seller']);

        if (error) {
        console.error('Error checking seller/admin status:', error);
        return false;
        }

        return data && data.length > 0;
    } catch (error) {
        console.error('Exception in isSellerOrAdmin check:', error);
        return false;
    }
    }

    /**
     * Check if a user is the owner of a product
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @param productId The ID of the product
     * @returns Boolean indicating if the user is the product owner
     */
    export async function isProductOwner(
    supabase: SupabaseClient,
    userId: string,
    productId: string
    ): Promise<boolean> {
    try {
        const { data, error } = await supabase
        .from('products')
        .select('seller_id')
        .eq('id', productId)
        .single();

        if (error || !data) {
        console.error('Error checking product ownership:', error);
        return false;
        }

        return data.seller_id === userId;
    } catch (error) {
        console.error('Exception in isProductOwner check:', error);
        return false;
    }
}