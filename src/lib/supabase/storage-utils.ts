import { supabase } from './client';

/**
 * Test the storage configuration to ensure it's properly set up
 */
export const testStorageConfiguration = async (): Promise<{
    success: boolean; 
    issues?: string[]; 
    error?: string;
}> => {
    const issues: string[] = [];
    
    try {
        const { data: buckets, error: bucketsError } = await supabase
            .storage
            .listBuckets();
    
        if (bucketsError) {
            throw new Error(`Error listing buckets: ${bucketsError.message}`);
        }
    
        const productImagesBucket = buckets?.find(bucket => bucket.name === 'product-images');
    
        if (!productImagesBucket) {
            issues.push('product-images bucket does not exist');
        }
    
        if (productImagesBucket) {
            const { error: fileListError } = await supabase
                .storage
                .from('product-images')
                .list();
            
            if (fileListError) {
                issues.push(`Cannot list files in bucket: ${fileListError.message}`);
            }
        }
    
        const publicUrl = `${process.env.NEXT_PUBLIC_SUPABASE_URL}/storage/v1/object/public/product-images/test-cors`;
    
        const testCorsResponse = await fetch(publicUrl, { method: 'OPTIONS' });
    
        if (!testCorsResponse.ok && testCorsResponse.status !== 204) {
            issues.push('CORS configuration may not be properly set up');
        }
    
        return { 
            success: issues.length === 0,
            issues: issues.length > 0 ? issues : undefined
        };
    } catch (error) {
        console.error('Error testing storage configuration:', error);
        return {
            success: false,
            error: error instanceof Error ? error.message : 'Unknown error occurred while testing storage',
            issues: issues.length > 0 ? issues : undefined
        };
    }
};