import { superAdmin } from '../lib/supabase/admin-client';

async function setupStorage() {
    try {
        const { error: bucketError } = await superAdmin
        .storage
        .createBucket('product-images', {
            public: true,
            allowedMimeTypes: ['image/jpeg', 'image/png', 'image/webp'],
            fileSizeLimit: 1024 * 1024 * 5 // 5MB
        });

        if (bucketError && bucketError.message !== 'Bucket already exists') {
        throw bucketError;
        }

        console.log('Bucket policies set successfully.');
    } catch (error) {
        console.error('Error setting up storage:', error);
    }
}

setupStorage(); 