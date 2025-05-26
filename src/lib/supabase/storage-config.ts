import { supabase } from './client';

/**
 * Configure Supabase storage buckets for the marketplace
 * This should be run once during initial setup
 */
export const configureStorage = async (): Promise<{ success: boolean; error?: string }> => {
    try {
        const { data: buckets, error: bucketsError } = await supabase
        .storage
        .listBuckets();
        
        if (bucketsError) {
        throw new Error(`Error listing buckets: ${bucketsError.message}`);
        }
        
        const productImagesBucket = buckets?.find(bucket => bucket.name === 'product-images');
        
        if (!productImagesBucket) {
            const { error: createError } = await supabase
                .storage
                .createBucket('product-images', {
                    public: true,
                    fileSizeLimit: 5242880, // 5MB limit
                    allowedMimeTypes: ['image/png', 'image/jpeg', 'image/gif', 'image/webp']
                });

            if (createError) {
                throw new Error(`Error creating product-images bucket: ${createError.message}`);
            }

            console.log('Created product-images bucket');
        }

        // Update the bucket to ensure it is publicly accessible
        const { error: updateError } = await supabase
            .storage
            .updateBucket('product-images', { public: true });

        if (updateError) {
            console.warn('Warning: Error updating bucket to public access:', updateError.message);
        }
    } catch (error) {
        if (error instanceof Error) {
            console.error('Error configuring storage:', error.message);
            return { success: false, error: error.message };
        } else {
            console.error('Error configuring storage:', error);
            return { success: false, error: String(error) };
        }
    }

    return { success: true };

// Note: Fine-grained access control (e.g., authenticated uploads) must be set up in the Supabase dashboard or via SQL.
}
