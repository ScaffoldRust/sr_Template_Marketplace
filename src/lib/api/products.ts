import { superAdmin } from '@/lib/supabase/admin-client';
import { PaginationParams, ProductFilter, Product, Database } from '../types/database.types';

export const productApi = {
    /**
     * Get all products with filtering, sorting, and pagination
     */
    async getProducts(
        pagination: PaginationParams = { page: 1, pageSize: 10 },
        filter: ProductFilter = {}
    ): Promise<{ data: Product[] | null; count: number | null; error: Error | null }> {
        const { page = 1, pageSize = 10 } = pagination;
        const offset = (page - 1) * pageSize;
        
        let query = superAdmin
        .from('products')
        .select('*', { count: 'exact' });
        
        if (filter.category) {
        query = query.eq('category', filter.category);
        }
        
        if (filter.minPrice !== undefined) {
        query = query.gte('price', filter.minPrice);
        }
        
        if (filter.maxPrice !== undefined) {
        query = query.lte('price', filter.maxPrice);
        }
        
        if (filter.minRating !== undefined) {
        query = query.gte('rating', filter.minRating);
        }
        
        if (filter.featured !== undefined) {
        query = query.eq('featured', filter.featured);
        }
        
        if (filter.sortBy) {
        const order = filter.sortOrder || 'asc';
        query = query.order(filter.sortBy, { ascending: order === 'asc' });
        } else {
        query = query.order('created_at', { ascending: false });
        }
        
        query = query.range(offset, offset + pageSize - 1);
        
        const { data, error, count } = await query;
        
        return { data, count, error: error as Error | null };
    },
    
    /**
     * Get product by ID
     */
    async getProductById(id: string): Promise<{ data: Product | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('products')
        .select('*')
        .eq('id', id)
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get product by slug
     */
    async getProductBySlug(slug: string): Promise<{ data: Product | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('products')
        .select('*')
        .eq('slug', slug)
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get products by category ID
     */
    async getProductsByCategory(
        categoryId: string,
        pagination: PaginationParams = { page: 1, pageSize: 10 }
    ): Promise<{ data: Product[] | null; count: number | null; error: Error | null }> {
        const { page = 1, pageSize = 10 } = pagination;
        const offset = (page - 1) * pageSize;
        
        const { data, error, count } = await superAdmin
        .from('products')
        .select('*', { count: 'exact' })
        .eq('category', categoryId)
        .range(offset, offset + pageSize - 1);
        
        return { data, count, error: error as Error | null };
    },
    
    /**
     * Get products by category slug
     */
    async getProductsByCategorySlug(
        categorySlug: string,
        pagination: PaginationParams = { page: 1, pageSize: 10 }
    ): Promise<{ data: Product[] | null; count: number | null; error: Error | null }> {
        const { data: category } = await superAdmin
        .from('categories')
        .select('id')
        .eq('slug', categorySlug)
        .single();
        
        if (!category) {
        return { data: null, count: null, error: new Error('Category not found') };
        }
        
        return await this.getProductsByCategory(category.id, pagination);
    },
    
    /**
     * Search products by title/description
     */
    async searchProducts(
        query: string,
        filter: ProductFilter = {},
        pagination: PaginationParams = { page: 1, pageSize: 10 }
    ): Promise<{ data: Product[] | null; error: Error | null }> {
        const { page = 1, pageSize = 10 } = pagination;
        
        const { data, error } = await superAdmin.rpc('search_products', {
        search_query: query,
        category_id: filter.category,
        min_price: filter.minPrice,
        max_price: filter.maxPrice,
        min_rating: filter.minRating,
        limit: pageSize,
        offset: (page - 1) * pageSize
        });
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get featured products
     */
    async getFeaturedProducts(limit: number = 8): Promise<{ data: Product[] | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('products')
        .select('*')
        .eq('featured', true)
        .order('rating', { ascending: false })
        .limit(limit);
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get related products based on category
     */
    async getRelatedProducts(productId: string, limit: number = 4): Promise<{ data: Product[] | null; error: Error | null }> {
        const { data: product } = await this.getProductById(productId);
        
        if (!product) {
        return { data: null, error: new Error('Product not found') };
        }
        
        const { data, error } = await superAdmin
        .from('products')
        .select('*')
        .eq('category', product.category)
        .neq('id', productId)
        .order('rating', { ascending: false })
        .limit(limit);
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Create a new product
     */
    async createProduct(product: Database['public']['Tables']['products']['Insert']): Promise<{ data: Product | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('products')
        .insert([product])
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Update an existing product
     */
    async updateProduct(id: string, updates: Database['public']['Tables']['products']['Update']): Promise<{ data: Product | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('products')
        .update(updates)
        .eq('id', id)
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Delete a product
     */
    async deleteProduct(id: string): Promise<{ error: Error | null }> {
        const { error } = await superAdmin
        .from('products')
        .delete()
        .eq('id', id);
        
        return { error: error as Error | null };
    }
};
