import { supabase } from '../supabase/client';
import { ProductImage } from '../types/database.types';

export type ImageUploadResult = {
    success: boolean;
    data?: ProductImage;
    error?: string;
};

export type ImageTransformOptions = {
    width?: number;
    height?: number;
    quality?: number;
    format?: 'webp' | 'jpeg' | 'png';
};

/**
 * Upload a product image to Supabase storage and create record in product_images table
 */
export const uploadProductImage = async (
    productId: string,
    file: File,
    altText: string,
    displayOrder: number = 0,
    isPrimary: boolean = false
): Promise<ImageUploadResult> => {
    try {
    const allowedTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/webp'];
    if (!allowedTypes.includes(file.type)) {
        return {
        success: false,
        error: `Unsupported file type. Please upload: ${allowedTypes.join(', ')}`
        };
    }
    
    const maxSize = 5 * 1024 * 1024;
    if (file.size > maxSize) {
        return {
        success: false,
        error: `File too large. Maximum size is 5MB.`
        };
    }
    
    const fileExt = file.name.split('.').pop();
    const timestamp = Date.now();
    const fileName = `${timestamp}.${fileExt}`;
    const filePath = `products/${productId}/${fileName}`;
    
    const { error: uploadError } = await supabase
        .storage
        .from('product-images')
        .upload(filePath, file);
    
    if (uploadError) {
        throw new Error(`Error uploading image: ${uploadError.message}`);
    }
    
    // Get public URL
    const { data: publicUrlData } = supabase
        .storage
        .from('product-images')
        .getPublicUrl(filePath);
    
    const url = publicUrlData.publicUrl;
    
    if (isPrimary) {
        await supabase
        .from('product_images')
        .update({ is_primary: false })
        .eq('product_id', productId)
        .eq('is_primary', true);
    }
    
    const { data, error } = await supabase
        .from('product_images')
        .insert([{
        product_id: productId,
        url,
        alt_text: altText,
        display_order: displayOrder,
        is_primary: isPrimary
        }])
        .select('*')
        .single();
    
    if (error) {
        await supabase
        .storage
        .from('product-images')
        .remove([filePath]);
        
        throw new Error(`Error creating image record: ${error.message}`);
    }
    
    return {
        success: true,
        data
    };
    } catch (error) {
    console.error('Error uploading product image:', error);
    return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred while uploading image'
    };
    }
};

/**
 * Delete a product image from storage and database
 */
export const deleteProductImage = async (
    imageId: string,
    productId: string
): Promise<{ success: boolean; error?: string }> => {
    try {
    const { data: image, error: fetchError } = await supabase
        .from('product_images')
        .select('url')
        .eq('id', imageId)
        .eq('product_id', productId)
        .single();
    
    if (fetchError) {
        throw new Error(`Error fetching image details: ${fetchError.message}`);
    }
    
    if (!image) {
        return {
        success: false,
        error: 'Image not found'
        };
    }
    
    // Extract the file path from the URL
    const url = new URL(image.url);
    const pathParts = url.pathname.split('/');
    const filePath = pathParts.slice(pathParts.indexOf('product-images') + 1).join('/');
    
    const { error: deleteError } = await supabase
        .from('product_images')
        .delete()
        .eq('id', imageId)
        .eq('product_id', productId);
    
    if (deleteError) {
        throw new Error(`Error deleting image record: ${deleteError.message}`);
    }
    
    const { error: storageError } = await supabase
        .storage
        .from('product-images')
        .remove([filePath]);
    
    if (storageError) {
        console.warn(`Warning: Deleted database record but failed to delete file: ${storageError.message}`);
    }
    
    const { data: remainingImages } = await supabase
        .from('product_images')
        .select('id')
        .eq('product_id', productId)
        .order('display_order', { ascending: true })
        .limit(1);
    
    if (remainingImages && remainingImages.length > 0) {
        await supabase
        .from('product_images')
        .update({ is_primary: true })
        .eq('id', remainingImages[0].id);
    }
    
    return { success: true };
    } catch (error) {
    console.error('Error deleting product image:', error);
    return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred while deleting image'
    };
    }
};

/**
 * Generate optimized image URLs with transformation parameters
 */
export const getTransformedImageUrl = (
    originalUrl: string,
    options: ImageTransformOptions
): string => {
    try {
    const url = new URL(originalUrl);
    
    if (options.width) url.searchParams.append('width', options.width.toString());
    if (options.height) url.searchParams.append('height', options.height.toString());
    if (options.quality) url.searchParams.append('quality', options.quality.toString());
    if (options.format) url.searchParams.append('format', options.format);
    
    return url.toString();
    } catch (error) {
    console.error('Error transforming image URL:', error);
    return originalUrl;
    }
};

/**
 * Update a product image's metadata
 */
export const updateProductImage = async (
    imageId: string,
    productId: string,
    updates: {
    alt_text?: string;
    display_order?: number;
    is_primary?: boolean;
    }
): Promise<ImageUploadResult> => {
    try {
    if (updates.is_primary) {
        await supabase
        .from('product_images')
        .update({ is_primary: false })
        .eq('product_id', productId)
        .eq('is_primary', true)
        .neq('id', imageId);
    }
    
    const { data, error } = await supabase
        .from('product_images')
        .update(updates)
        .eq('id', imageId)
        .eq('product_id', productId)
        .select('*')
        .single();
    
    if (error) {
        throw new Error(`Error updating image: ${error.message}`);
    }
    
    return {
        success: true,
        data
    };
    } catch (error) {
    console.error('Error updating product image:', error);
    return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred while updating image'
    };
    }
};

/**
 * Get all images for a product
 */
export const getProductImages = async (
    productId: string
): Promise<{ success: boolean; data?: ProductImage[]; error?: string }> => {
    try {
    const { data, error } = await supabase
        .from('product_images')
        .select('*')
        .eq('product_id', productId)
        .order('display_order', { ascending: true });
    
    if (error) {
        throw new Error(`Error fetching product images: ${error.message}`);
    }
    
    return {
        success: true,
        data
    };
    } catch (error) {
    console.error('Error getting product images:', error);
    return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred while fetching images'
    };
    }
};

/**
 * Reorder product images
 */
export const reorderProductImages = async (
    productId: string,
    imageOrders: Array<{ id: string; displayOrder: number }>
): Promise<{ success: boolean; error?: string }> => {
    try {
    const updatePromises = imageOrders.map(({ id, displayOrder }) => {
        return supabase
        .from('product_images')
        .update({ display_order: displayOrder })
        .eq('id', id)
        .eq('product_id', productId); // Security check
    });
    
    const results = await Promise.all(updatePromises);
    
    const errors = results
        .filter(result => result.error)
        .map(result => result.error?.message);
    
    if (errors.length > 0) {
        throw new Error(`Errors reordering images: ${errors.join(', ')}`);
    }
    
    return { success: true };
    } catch (error) {
    console.error('Error reordering product images:', error);
    return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred while reordering images'
    };
    }
};