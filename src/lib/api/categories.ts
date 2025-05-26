import { superAdmin } from '@/lib/supabase/admin-client';
import { Category, Database } from '../types/database.types';

export const categoryApi = {
    /**
     * Get all categories
     */
    async getCategories(): Promise<{ data: Category[] | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('categories')
        .select('*')
        .order('name');
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get category by ID
     */
    async getCategoryById(id: string): Promise<{ data: Category | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('categories')
        .select('*')
        .eq('id', id)
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get category by slug
     */
    async getCategoryBySlug(slug: string): Promise<{ data: Category | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('categories')
        .select('*')
        .eq('slug', slug)
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get category with subcategories
     */
    async getCategoryWithSubcategories(parentId: string | null = null): Promise<{ data: Category[] | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('categories')
        .select('*')
        .eq('parent_id', parentId)
        .order('name');
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Create a new category
     */
    async createCategory(category: Database['public']['Tables']['categories']['Insert']): Promise<{ data: Category | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('categories')
        .insert([category])
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Update an existing category
     */
    async updateCategory(id: string, updates: Database['public']['Tables']['categories']['Update']): Promise<{ data: Category | null; error: Error | null }> {
        const { data, error } = await superAdmin
        .from('categories')
        .update(updates)
        .eq('id', id)
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Delete a category
     */
    async deleteCategory(id: string): Promise<{ error: Error | null }> {
        const { error } = await superAdmin
        .from('categories')
        .delete()
        .eq('id', id);
        
        return { error: error as Error | null };
    }
};