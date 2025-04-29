import { SupabaseClient } from '@supabase/supabase-js';
import { ProductRating, RatingAddRequest } from '../types';
import { createErrorResponse, ErrorCode, handleGenericError } from '../utils/error-handlers';
import { validateRatingAddRequest } from '../utils/validators';

    /**
     * Add a rating to a product
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @param request The rating request
     * @returns The newly created rating
     */

    interface User {
        full_name: string;
    }

    export async function addRating(
    supabase: SupabaseClient,
    userId: string,
    request: RatingAddRequest
    ): Promise<ProductRating> {
    try {
        const validationError = validateRatingAddRequest(request);
        if (validationError) {
            throw createErrorResponse(validationError, ErrorCode.VALIDATION_ERROR);
        }

        const { data: product, error: productError } = await supabase
        .from('products')
        .select('id')
        .eq('id', request.productId)
        .single();

        if (productError || !product) {
            throw createErrorResponse('Product not found', ErrorCode.NOT_FOUND);
        }

        // Check if the user has already rated this product
        const { data: existingRating, error: ratingError } = await supabase
        .from('product_ratings')
        .select('id')
        .eq('product_id', request.productId)
        .eq('user_id', userId)
        .maybeSingle();

        if (ratingError && ratingError.code !== 'PGRST116') {
            throw ratingError;
        }

        let result;

        if (existingRating) {
            const { data, error } = await supabase
            .from('product_ratings')
            .update({
            rating: request.rating,
            comment: request.comment,
            })
            .eq('id', existingRating.id)
            .select('*')
            .single();

            if (error || !data) {
                throw error || new Error('Failed to update rating');
            }

            result = data;
        } else {
            const { data, error } = await supabase
            .from('product_ratings')
            .insert({
                product_id: request.productId,
                user_id: userId,
                rating: request.rating,
                comment: request.comment,
            })
            .select('*')
            .single();

            if (error || !data) {
                throw error || new Error('Failed to add rating');
            }

            result = data;
        }

        // Update the product's aggregated rating
        await updateProductRating(supabase, request.productId);

        return result;    
        } catch (error) {
            console.error('Error adding rating:', error);
            throw error instanceof Error && 'code' in error 
            ? error 
            : handleGenericError(error);
        }
    }

    /**
     * Delete a rating
     * @param supabase The Supabase client
     * @param userId The ID of the user
     * @param ratingId The ID of the rating
     */
    export async function deleteRating(
    supabase: SupabaseClient,
    userId: string,
    ratingId: string
    ): Promise<void> {
    try {
        const { data: rating, error: findError } = await supabase
        .from('product_ratings')
        .select('id, product_id')
        .eq('id', ratingId)
        .eq('user_id', userId)
        .single();

        if (findError || !rating) {
            throw createErrorResponse('Rating not found', ErrorCode.NOT_FOUND);
        }

        const { error: deleteError } = await supabase
        .from('product_ratings')
        .delete()
        .eq('id', ratingId);

        if (deleteError) {
            throw deleteError;
        }

        await updateProductRating(supabase, rating.product_id);
        } catch (error) {
            console.error('Error deleting rating:', error);
            throw error instanceof Error && 'code' in error 
            ? error 
            : handleGenericError(error);
        }
    }

    /**
     * Get ratings for a product
     * @param supabase The Supabase client
     * @param productId The ID of the product
     * @param page The page number
     * @param limit The number of ratings per page
     * @returns List of ratings with pagination
     */
    export async function getProductRatings(
    supabase: SupabaseClient,
    productId: string,
    page: number = 1,
    limit: number = 10
    ): Promise<{ ratings: ProductRating[]; total: number; page: number; limit: number }> {
        try {
            // Validate pagination parameters
            if (page < 1) page = 1;
            if (limit < 1) limit = 10;
            if (limit > 50) limit = 50;

            const offset = (page - 1) * limit;

            // Get the ratings with user information
            const { data, error, count } = await supabase
            .from('product_ratings')
            .select(`
                id,
                product_id,
                user_id,
                rating,
                comment,
                created_at,
                users(full_name, avatar_url)
            `, { count: 'exact' })
            .eq('product_id', productId)
            .order('created_at', { ascending: false })
            .range(offset, offset + limit - 1);

            if (error) {
                throw error;
            }

            // Format the ratings
            const ratings = data.map((rating) => {
            const user = Array.isArray(rating.users) && rating.users.length > 0 
                ? rating.users[0] as User 
                : null;
                return {
                    id: rating.id,
                    product_id: rating.product_id,
                    user_id: rating.user_id,
                    rating: rating.rating,
                    comment: rating.comment,
                    created_at: rating.created_at,
                    user: user ? {
                    full_name: user.full_name,
                    } : undefined,
                };
            });

            return {
            ratings,
            total: count || 0,
            page,
            limit,
            };
        } catch (error) {
            console.error('Error getting product ratings:', error);
            throw handleGenericError(error);
        }
    }

    /**
     * Update a product's aggregated rating
     * @param supabase The Supabase client
     * @param productId The ID of the product
     */
    export async function updateProductRating(
    supabase: SupabaseClient,
    productId: string
    ): Promise<void> {
    try {
        const { data, error } = await supabase
        .rpc('calculate_product_rating', { product_id_param: productId });

        if (error) {
        throw error;
        }

        if (!data || data.length === 0) {
        // No ratings found, set default values
        const { error: updateError } = await supabase
            .from('products')
            .update({
            rating: 0,
            rating_count: 0,
            })
            .eq('id', productId);

        if (updateError) {
            throw updateError;
        }
        return;
        }

        const result = data[0];
        
        // Update the product with the new rating and count
        const { error: updateError } = await supabase
        .from('products')
        .update({
            rating: result.avg_rating || 0,
            rating_count: result.count || 0,
        })
        .eq('id', productId);

        if (updateError) {
        throw updateError;
        }
    } catch (error) {
        console.error('Error updating product rating:', error);
        throw handleGenericError(error);
    }
}

/**
 * Check if a user has rated a product
 * @param supabase The Supabase client
 * @param userId The ID of the user
 * @param productId The ID of the product
 * @returns The rating if it exists, null otherwise
 */
// export async function getUserProductRating(
//     supabase: SupabaseClient,
//     userId: string,
//     productId: string
//     ): Promise<ProductRating | null> {
//     try {
//         const { data, error } = await supabase
//         .from('product_ratings')
//         .select('*')
//         .eq('product_id', productId)
//         .eq('user_id', userId)
//         .single();

//         if (error) {
//         if (error.code === 'PGRST116') {
//             // "No rows returned" error, user hasn't rated this product
//             return null;
//         }
//         throw error;
//         }

//         return data;
//     } catch (error) {
//         console.error('Error getting user product rating:', error);
//         throw handleGenericError(error);
//     }
// }