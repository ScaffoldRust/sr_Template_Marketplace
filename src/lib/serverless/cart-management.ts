import { SupabaseClient } from '@supabase/supabase-js';
import { CartAddRequest, CartItem, CartResponse, CartUpdateRequest } from '../types';
import { createErrorResponse, ErrorCode, handleGenericError } from '../utils/error-handlers';
import { Product } from '../types/database.types';
import { validateCartAddRequest } from '../utils/validators';

/**
 * Add a product to the user's cart
 * @param supabase The Supabase client
 * @param userId The ID of the user
 * @param request The cart add request
 * @returns The updated cart
 */
export async function addToCart(
    supabase: SupabaseClient,
    userId: string,
    request: CartAddRequest
    ): Promise<CartResponse> {
    try {
        const validationError = validateCartAddRequest(request);
        if (validationError) {
            throw createErrorResponse(validationError, ErrorCode.VALIDATION_ERROR);
        }

        const { data: product, error: productError } = await supabase
        .from('products')
        .select('id, stock')
        .eq('id', request.productId)
        .single();

        if (productError || !product) {
            throw createErrorResponse('Product not found', ErrorCode.NOT_FOUND);
        }

        if (product.stock < request.quantity) {
            throw createErrorResponse(
                `Not enough stock. Only ${product.stock} items available.`,
                ErrorCode.BAD_REQUEST
            );
        }

        const { data: existingItem, error: existingError } = await supabase
        .from('cart_items')
        .select('id, quantity')
        .eq('user_id', userId)
        .eq('product_id', request.productId)
        .single();

        if (!existingError && existingItem) {
        const newQuantity = existingItem.quantity + request.quantity;
        
        if (newQuantity > product.stock) {
            throw createErrorResponse(
                `Cannot add more items. Only ${product.stock} items available in total.`,
                ErrorCode.BAD_REQUEST
            );
        }

        const { error: updateError } = await supabase
            .from('cart_items')
            .update({ quantity: newQuantity, updated_at: new Date().toISOString() })
            .eq('id', existingItem.id);

        if (updateError) {
            throw updateError;
        }
        } else {
        const { data: newItem, error: insertError } = await supabase
            .from('cart_items')
            .insert({
                user_id: userId,
                product_id: request.productId,
                quantity: request.quantity,
            })
            .select('id')
            .single();

        if (insertError || !newItem) {
            throw insertError || new Error('Failed to add item to cart');
        }
        }

        return await getCart(supabase, userId);
    } catch (error) {
        console.error('Error adding to cart:', error);
        throw error instanceof Error && 'code' in error 
        ? error 
        : handleGenericError(error);
    }
    }

    /**
     * Update the quantity of a cart item
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @param cartItemId The ID of the cart item
     * @param request The cart update request
     * @returns The updated cart
     */
    export async function updateCartItem(
    supabase: SupabaseClient,
    userId: string,
    cartItemId: string,
    request: CartUpdateRequest
    ): Promise<CartResponse> {
    try {
        if (typeof request.quantity !== 'number' || request.quantity < 1) {
        throw createErrorResponse('Quantity must be a positive number', ErrorCode.VALIDATION_ERROR);
        }

        const { data: cartItem, error: cartItemError } = await supabase
        .from('cart_items')
        .select('id, product_id')
        .eq('id', cartItemId)
        .eq('user_id', userId)
        .single();

        if (cartItemError || !cartItem) {
        throw createErrorResponse('Cart item not found', ErrorCode.NOT_FOUND);
        }

        const { data: product, error: productError } = await supabase
        .from('products')
        .select('stock')
        .eq('id', cartItem.product_id)
        .single();

        if (productError || !product) {
        throw createErrorResponse('Product not found', ErrorCode.NOT_FOUND);
        }

        if (product.stock < request.quantity) {
        throw createErrorResponse(
            `Not enough stock. Only ${product.stock} items available.`,
            ErrorCode.BAD_REQUEST
        );
        }

        const { error: updateError } = await supabase
        .from('cart_items')
        .update({
            quantity: request.quantity,
            updated_at: new Date().toISOString(),
        })
        .eq('id', cartItemId);

        if (updateError) {
        throw updateError;
        }

        return await getCart(supabase, userId);
    } catch (error) {
        console.error('Error updating cart item:', error);
        throw error instanceof Error && 'code' in error 
        ? error 
        : handleGenericError(error);
    }
    }

    /**
     * Remove an item from the cart
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @param cartItemId The ID of the cart item
     * @returns The updated cart
     */
    export async function removeFromCart(
    supabase: SupabaseClient,
    userId: string,
    cartItemId: string
    ): Promise<CartResponse> {
    try {
        const { data: cartItem, error: cartItemError } = await supabase
        .from('cart_items')
        .select('id')
        .eq('id', cartItemId)
        .eq('user_id', userId)
        .single();

        if (cartItemError || !cartItem) {
        throw createErrorResponse('Cart item not found', ErrorCode.NOT_FOUND);
        }

        const { error: deleteError } = await supabase
        .from('cart_items')
        .delete()
        .eq('id', cartItemId);

        if (deleteError) {
        throw deleteError;
        }

        return await getCart(supabase, userId);
    } catch (error) {
        console.error('Error removing from cart:', error);
        throw error instanceof Error && 'code' in error 
        ? error 
        : handleGenericError(error);
    }
    }

    /**
     * Clear the user's cart
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @returns The empty cart
     */
    export async function clearCart(supabase: SupabaseClient, userId: string): Promise<CartResponse> {
    try {
        const { error: deleteError } = await supabase
        .from('cart_items')
        .delete()
        .eq('user_id', userId);

        if (deleteError) {
        throw deleteError;
        }

        return {
        items: [],
        total: 0,
        itemCount: 0,
        };
    } catch (error) {
        console.error('Error clearing cart:', error);
        throw error instanceof Error && 'code' in error 
        ? error 
        : handleGenericError(error);
    }
    }

    /**
     * Get the user's cart
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @returns The cart with items and total
     */
    export async function getCart(supabase: SupabaseClient, userId: string): Promise<CartResponse> {
    try {
        const { data: cartItems, error: cartError } = await supabase
        .from('cart_items')
        .select(`
            id,
            user_id,
            product_id,
            quantity,
            created_at,
            updated_at,
            product:products(
            id,
            title,
            price,
            stock,
            slug,
            rating,
            images:product_images(url, is_primary)
            )
        `)
        .eq('user_id', userId);

        if (cartError) {
        throw cartError;
        }

        let total = 0;
        let itemCount = 0;

        const items: CartItem[] = cartItems.map((item) => {
            const product = item.product as unknown as Product;
            const price = product?.price || 0;
            total += price * item.quantity;
            itemCount += item.quantity;

            return {
                id: item.id,
                user_id: item.user_id,
                product_id: item.product_id,
                quantity: item.quantity,
                created_at: item.created_at,
                updated_at: item.updated_at,
                product: product ? {
                id: product.id,
                title: product.title,
                price: product.price,
                stock: product.stock,
                slug: product.slug,
                rating: product.rating,
                featured: product.featured,
                } : undefined,
            };
        });

        return {
        items,
        total,
        itemCount,
        };
    } catch (error) {
        console.error('Error getting cart:', error);
        throw handleGenericError(error);
    }
    }

    /**
     * Apply a promotion code to the cart
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @param promoCode The promotion code
     * @returns The updated cart with the promotion applied
     */
    export async function applyPromoCode(
    supabase: SupabaseClient,
    userId: string,
    promoCode: string
    ): Promise<CartResponse & { discount: number }> {
    try {
        const { data: promotion, error: promoError } = await supabase
        .from('promotions')
        .select('id, code, discount_percentage, active')
        .eq('code', promoCode.toUpperCase())
        .eq('active', true)
        .single();

        if (promoError || !promotion) {
        throw createErrorResponse('Invalid or expired promotion code', ErrorCode.NOT_FOUND);
        }

        const cart = await getCart(supabase, userId);

        const discountMultiplier = promotion.discount_percentage / 100;
        const discountAmount = cart.total * discountMultiplier;
        const discountedTotal = cart.total - discountAmount;

        return {
        ...cart,
        total: discountedTotal,
        discount: discountAmount,
        };
    } catch (error) {
        console.error('Error applying promo code:', error);
        throw error instanceof Error && 'code' in error 
        ? error 
        : handleGenericError(error);
    }
}