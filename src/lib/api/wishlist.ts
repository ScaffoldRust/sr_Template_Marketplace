import { supabase } from '../supabase/client';
import { WishlistItem } from '../types/database.types';

export const wishlistApi = {
    /**
     * Get user's wishlist with product details
     */
    async getUserWishlist(userId: string): Promise<{ data: WishlistItem[] | null; error: Error | null }> {
        const { data, error } = await supabase
        .from('wishlist_items')
        .select(`
            id,
            user_id,
            product_id,
            created_at,
            products (*)
        `)
        .eq('user_id', userId);
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Add product to wishlist
     */
    async addToWishlist(userId: string, productId: string): Promise<{ data: WishlistItem | null; error: Error | null }> {
        const { data, error } = await supabase
        .from('wishlist_items')
        .insert([{ user_id: userId, product_id: productId }])
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Remove product from wishlist
     */
    async removeFromWishlist(userId: string, productId: string): Promise<{ error: Error | null }> {
        const { error } = await supabase
        .from('wishlist_items')
        .delete()
        .eq('user_id', userId)
        .eq('product_id', productId);
        
        return { error: error as Error | null };
    },
    
    /**
     * Check if product is in wishlist
     */
    async isInWishlist(userId: string, productId: string): Promise<{ isInWishlist: boolean; error: Error | null }> {
        const { data, error } = await supabase
        .from('wishlist_items')
        .select('id')
        .eq('user_id', userId)
        .eq('product_id', productId)
        .maybeSingle();
        
        return { isInWishlist: !!data, error: error as Error | null };
    }
};
