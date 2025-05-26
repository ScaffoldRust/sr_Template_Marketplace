import { SupabaseClient } from '@supabase/supabase-js';
import { z } from "zod";
import { ProductSearchParams, SearchResponse } from '../types';
import { handleGenericError } from '../utils/error-handlers';
import { validateSearchParams, validatePagination } from '../utils/validators';

/**
 * Search for products with various filters and sorting options
 * @param supabase The Supabase client
 * @param params Search parameters
 * @returns Filtered and paginated product results
 */
export async function searchProducts(
    supabase: SupabaseClient,
    params: ProductSearchParams
    ): Promise<SearchResponse> {
    try {
        const validationError = validateSearchParams(params);
        if (validationError) {
        throw new Error(validationError);
        }

        const { page, limit } = validatePagination(params.page, params.limit);
        const offset = (page - 1) * limit;

        let query = supabase
        .from('products')
        .select(`
            id,
            title,
            description,
            price,
            category,
            rating,
            rating_count,
            seller_id,
            created_at,
            updated_at,
            stock,
            slug,
            featured,
            category_details:categories!inner(name, slug),
            images:product_images(id, url, alt_text, is_primary)
        `, { count: 'exact' });

        if (params.query && params.query.trim() !== '') {
        const searchTerm = params.query.trim();
        query = query.textSearch('fts', searchTerm, {
            type: 'websearch',
            config: 'english',
        });
        }

        if (params.category) {
            if (params.category.includes('-')) {
                query = query.eq('categories.slug', params.category);
            } else {
                // Try as a category ID
                query = query.eq('category', params.category);
            }
        }

        if (params.minPrice !== undefined) {
            query = query.gte('price', params.minPrice);
        }

        if (params.maxPrice !== undefined) {
            query = query.lte('price', params.maxPrice);
        }

        if (params.minRating !== undefined) {
            query = query.gte('rating', params.minRating);
        }

        if (params.featured !== undefined) {
            query = query.eq('featured', params.featured);
        }

        const sortBy = params.sortBy || 'created_at';
        const validSortFields = [
            'title',
            'price',
            'created_at',
            'updated_at',
            'rating',
            'rating_count',
        ];
        const direction = params.sortDirection === 'asc' ? 'asc' : 'desc';

        if (validSortFields.includes(sortBy)) {
            query = query.order(sortBy, { ascending: direction === 'asc' });
        } else {
            query = query.order('created_at', { ascending: false });
        }

        query = query.range(offset, offset + limit - 1);

        const { data, error, count } = await query;

        if (error) {
        throw error;
        }

        // Process the results
        const products = data.map((product) => {
            const categoryDetails = Array.isArray(product.category_details) && product.category_details.length > 0
                ? product.category_details[0]
                : undefined;
        
            return {
                ...product,
                category_details: categoryDetails ? {
                name: categoryDetails.name,
                slug: categoryDetails.slug,
                } : undefined,
            };
        });

        // Calculate total pages
        const totalCount = count || 0;
        const totalPages = Math.ceil(totalCount / limit);

        return {
            products,
            total: totalCount,
            page,
            limit,
            totalPages,
        };
        } catch (error) {
            console.error('Error searching products:', error);
            throw handleGenericError(error);
        }
    }

    /**
     * Get related products based on category and tags
     * @param supabase The Supabase client
     * @param productId The ID of the current product
     * @param limit The maximum number of related products to return
     * @returns List of related products
     */
    // Define Zod schema matching Product type
    const ProductSchema = z.object({
        id: z.string(),
        title: z.string(),
        price: z.number(),
        slug: z.string(),
        rating: z.number().optional(),
        description: z.string().optional(),
        category: z.string(),
        rating_count: z.number().optional(),
        seller_id: z.string(),
        created_at: z.string().datetime().optional(),
        updated_at: z.string().datetime().optional(),
        stock: z.number().optional(),
        featured: z.boolean().optional(),
    });
    
    type Product = z.infer<typeof ProductSchema>;
    
    export async function getRelatedProducts(
        supabase: SupabaseClient,
        productId: string,
        limit: number = 4
    ): Promise<Product[]> {
        try {
        const { data: product, error: productError } = await supabase
            .from("products")
            .select("category")
            .eq("id", productId)
            .single();
    
        if (productError || !product) {
            throw productError || new Error("Product not found");
        }
    
        const { data: relatedProducts, error: relatedError } = await supabase
            .from("products")
            .select(`
            id,
            title,
            price,
            slug,
            rating,
            description,
            category,
            rating_count,
            seller_id,
            created_at,
            updated_at,
            stock,
            featured
            `)
            .eq("category", product.category)
            .neq("id", productId)
            .order("rating", { ascending: false })
            .limit(limit);
    
        if (relatedError) {
            throw relatedError;
        }
    
        const result = z.array(ProductSchema).safeParse(relatedProducts);
        
        if (!result.success) {
            console.error("Data validation failed:", result.error.format());
            return [];
        }
    
        return result.data;
        
        } catch (error) {
        console.error("Error in getRelatedProducts:", error);
        throw handleGenericError(error);
        }
    }

    /**
     * Get featured products
     * @param supabase The Supabase client
     * @param limit The maximum number of featured products to return
     * @returns List of featured products
     */
    export async function getFeaturedProducts(
        supabase: SupabaseClient,
        limit: number = 8
    ): Promise<Product[]> {
    try {
        const { data, error } = await supabase
            .from('products')
            .select(`
            id,
            title,
            price,
            slug,
            rating,
            category,
            stock,
            featured,
            description,
            rating_count,
            seller_id,
            created_at,
            updated_at,
            category_details:categories(name, slug),
            images:product_images(url, alt_text, is_primary)
            `)
            .eq('featured', true)
            .order('created_at', { ascending: false })
            .limit(limit);
    
        if (error) {
            throw error;
        }
    
        // Transform the data to match the Product type
        const products: Product[] = (data || []).map((item) => ({
            id: item.id,
            title: item.title,
            description: item.description,
            price: item.price,
            category: item.category,
            rating: item.rating || 0,
            rating_count: item.rating_count || 0, 
            seller_id: item.seller_id ,
            created_at: item.created_at || new Date().toISOString(),
            updated_at: item.updated_at || new Date().toISOString(),
            stock: item.stock || 0,
            slug: item.slug,
            featured: item.featured || false,
        }));
    
        return products;
    } catch (error) {
        console.error('Error getting featured products:', error);
        throw handleGenericError(error);
    }
}

