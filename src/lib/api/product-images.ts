import { supabase } from '../supabase/client';
import { ProductImage } from '../types/database.types';

export const productImageApi = {
    /**
     * Get all images for a product
     */
    async getProductImages(productId: string): Promise<{ data: ProductImage[] | null; error: Error | null }> {
        const { data, error } = await supabase
        .from('product_images')
        .select('*')
        .eq('product_id', productId)
        .order('display_order');
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Get primary image for a product
     */
    async getProductPrimaryImage(productId: string): Promise<{ data: ProductImage | null; error: Error | null }> {
        const { data, error } = await supabase
        .from('product_images')
        .select('*')
        .eq('product_id', productId)
        .eq('is_primary', true)
        .maybeSingle();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Upload a product image
     */
    async uploadProductImage(
        productId: string, 
        file: File, 
        altText: string, 
        displayOrder: number = 0, 
        isPrimary: boolean = false
    ): Promise<{ data: ProductImage | null; error: Error | null }> {
        const fileExt = file.name.split('.').pop();
        const fileName = `${productId}/${Date.now()}.${fileExt}`;
        const filePath = `products/${fileName}`;
        
        const { error: uploadError } = await supabase
        .storage
        .from('product-images')
        .upload(filePath, file);
        
        if (uploadError) {
        return { data: null, error: uploadError as Error };
        }
        
        const { data: publicUrlData } = supabase
        .storage
        .from('product-images')
        .getPublicUrl(filePath);
        
        const url = publicUrlData.publicUrl;
        
        const { data, error } = await supabase
        .from('product_images')
        .insert([{
            product_id: productId,
            url,
            alt_text: altText,
            display_order: displayOrder,
            is_primary: isPrimary
        }])
        .select()
        .single();
        
        return { data, error: error as Error | null };
    },
    
    /**
     * Delete a product image
     */
    async deleteProductImage(imageId: string): Promise<{ error: Error | null }> {
        const { data: image } = await supabase
        .from('product_images')
        .select('url')
        .eq('id', imageId)
        .single();
        
        if (image) {
        const path = image.url.split('/').slice(-2).join('/');
        
        await supabase
            .storage
            .from('product-images')
            .remove([`products/${path}`]);
        }
        
        const { error } = await supabase
        .from('product_images')
        .delete()
        .eq('id', imageId);
        
        return { error: error as Error | null };
    },
    
    /**
     * Update image details
     */
    async updateProductImage(
        imageId: string, 
        updates: Partial<Omit<ProductImage, 'id' | 'product_id' | 'url'>>
    ): Promise<{ data: ProductImage | null; error: Error | null }> {
        const { data, error } = await supabase
        .from('product_images')
        .update(updates)
        .eq('id', imageId)
        .select()
        .single();
        
        return { data, error: error as Error | null };
    }
};