import { uploadProductImage } from './image-handler';

export type ImageMigrationResult = {
    success: boolean;
    totalProcessed: number;
    successCount: number;
    failureCount: number;
    errors?: Array<{ productId: string; url: string; error: string }>;
};

/**
 * Migrate product images from external URLs to Supabase storage
 * Useful for initial data seeding or migrating from another platform
 */
export const migrateProductImages = async (
    imageDataList: Array<{
        productId: string;
        externalUrl: string;
        altText: string;
        displayOrder: number;
        isPrimary: boolean;
    }>
): Promise<ImageMigrationResult> => {
    const result: ImageMigrationResult = {
        success: false,
        totalProcessed: imageDataList.length,
        successCount: 0,
        failureCount: 0,
        errors: []
    };
    
        try {
            for (const imageData of imageDataList) {
        try {
            const response = await fetch(imageData.externalUrl);
            
            if (!response.ok) {
                throw new Error(`Failed to fetch image: ${response.statusText}`);
            }
            
            const contentType = response.headers.get('content-type') || 'image/jpeg';
            const blob = await response.blob();
            
            const fileName = imageData.externalUrl.split('/').pop() || 'image.jpg';
            const file = new File([blob], fileName, { type: contentType });
            
            const uploadResult = await uploadProductImage(
                imageData.productId,
                file,
                imageData.altText,
                imageData.displayOrder,
                imageData.isPrimary
            );
            
            if (!uploadResult.success) {
                throw new Error(uploadResult.error);
            }
            
            result.successCount++;
            } catch (error) {
            result.failureCount++;
            result.errors = result.errors || [];
            result.errors.push({
                productId: imageData.productId,
                url: imageData.externalUrl,
                error: error instanceof Error ? error.message : 'Unknown error'
            });
        }
    }
    
    result.success = result.successCount > 0;
    return result;
    } catch (error) {
    console.error('Error in bulk image migration:', error);
    return {
            ...result,
            success: false,
            errors: [
                ...(result.errors || []),
                { productId: '', url: '', error: error instanceof Error ? error.message : 'Unknown error occurred during migration' }
            ]
        };
    }
};