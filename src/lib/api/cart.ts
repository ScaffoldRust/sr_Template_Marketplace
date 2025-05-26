import { supabase } from '../supabase/client';
import { CartItem } from '../types/database.types';

export const cartApi = {
    /**
     * Get user's cart with product details
     */
    async getCart(userId: string): Promise<{ data: CartItem[] | null; error: Error | null }> {
        const { data, error } = await supabase
        .from('cart_items')
        .select(`
            id,
            user_id,
            product_id,
            quantity,
            created_at,
            updated_at,
            products (*)
        `)
        .eq('user_id', userId);
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Add product to cart
     */
    async addToCart(userId: string, productId: string, quantity: number = 1): Promise<{ data: CartItem | null; error: Error | null }> {
        const { data: existingItem } = await supabase
        .from('cart_items')
        .select('id, quantity')
        .eq('user_id', userId)
        .eq('product_id', productId)
        .maybeSingle();
        
        if (existingItem) {
        const newQuantity = existingItem.quantity + quantity;
        return await this.updateCartItemQuantity(userId, productId, newQuantity);
        } else {
            const { data, error } = await supabase
                .from('cart_items')
                .insert([{ user_id: userId, product_id: productId, quantity }])
                .select()
                .single();
                
        
            return { data, error: error as Error | null };
        }
    },
    
    /**
     * Update cart item quantity
     */
    async updateCartItemQuantity(userId: string, productId: string, quantity: number): Promise<{ data: CartItem | null; error: Error | null }> {
        const { data: product } = await supabase
        .from('products')
        .select('stock')
        .eq('id', productId)
        .single();
        
        if (!product || product.stock < quantity) {
        return { 
            data: null, 
            error: new Error(`Only ${product?.stock || 0} items available in stock`)
        };
        }
        
        const { data, error } = await supabase
        .from('cart_items')
        .update({ quantity, updated_at: new Date().toISOString() })
        .eq('user_id', userId)
        .eq('product_id', productId)
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Remove item from cart
     */
    async removeFromCart(userId: string, productId: string): Promise<{ error: Error | null }> {
        const { error } = await supabase
        .from('cart_items')
        .delete()
        .eq('user_id', userId)
        .eq('product_id', productId);
        
        return { error: error as Error | null };
    },
    
    /**
     * Clear cart (remove all items)
     */
    async clearCart(userId: string): Promise<{ error: Error | null }> {
        const { error } = await supabase
        .from('cart_items')
        .delete()
        .eq('user_id', userId);
        
        return { error: error as Error | null };
    },
    
    /**
     * Calculate cart total
     */
    async calculateCartTotal(userId: string): Promise<{ total: number; error: Error | null }> {
        const { data, error } = await supabase.rpc('calculate_cart_total', {
        user_id: userId
        });
        
        if (error) {
        return { total: 0, error: error as Error | null };
        }
        
        return { total: data.total || 0, error: null };
    }
};
