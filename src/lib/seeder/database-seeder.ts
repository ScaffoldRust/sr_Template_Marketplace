import { supabase } from '../supabase/client';
import { Category, Product } from '../types/database.types';

/**
 * Seed the database with initial categories and products
 */
export const seedDatabase = async (): Promise<{ success: boolean; error?: string }> => {
    try {
    // Seed categories first
    const categories = await seedCategories();
    if (!categories.success) {
        return { success: false, error: categories.error };
    }
    
    // Then seed products
    if (!categories.data) {
        return { success: false, error: 'Categories data is undefined' };
    }
    const products = await seedProducts(categories.data);
    if (!products.success) {
        return { success: false, error: products.error };
    }
    
    return { success: true };
    } catch (error) {
    console.error('Error seeding database:', error);
    return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Unknown error occurred while seeding database'
    };
    }
};

/**
 * Seed initial categories
 */
const seedCategories = async (): Promise<{ success: boolean; data?: Category[]; error?: string }> => {
    try {
    // Define initial categories
    const initialCategories = [
        {
            name: 'Clothing',
            slug: 'clothing',
            description: 'Apparel and wearable items'
        },
        {
            name: 'Electronics',
            slug: 'electronics',
            description: 'Electronic devices and accessories'
        },
        {
            name: 'Books',
            slug: 'books',
            description: 'Printed and digital books'
        },
        {
            name: 'Art',
            slug: 'art',
            description: 'Artwork and creative pieces'
        },
            {
            name: 'Virtual Goods',
            slug: 'virtual-goods',
            description: 'Digital items and virtual assets'
        },
        {
            name: 'Collectibles',
            slug: 'collectibles',
            description: 'Collectible items and memorabilia'
        }
    ];
    
    // Check if categories already exist
    const { data: existingCategories } = await supabase
        .from('categories')
        .select('*');
    
    if (existingCategories && existingCategories.length > 0) {
        console.log('Categories already exist, skipping seed.');
        return { success: true, data: existingCategories };
    }
    
    // Insert categories
    const { data, error } = await supabase
    .from('categories')
    .insert(initialCategories)
    .select();
    
    if (error) {
        throw new Error(`Error inserting categories: ${error.message}`);
    }
    
    // Add subcategories
    const subCategories = [
      // Clothing subcategories
        {
            name: 'T-Shirts',
            slug: 't-shirts',
            description: 'Short-sleeved casual tops',
            parent_id: data.find(c => c.slug === 'clothing')?.id
        },
        {
            name: 'Hoodies',
            slug: 'hoodies',
            description: 'Sweatshirts with hoods',
            parent_id: data.find(c => c.slug === 'clothing')?.id
        },
        
      // Electronics subcategories
        {
            name: 'Smartphones',
            slug: 'smartphones',
            description: 'Mobile phones and accessories',
            parent_id: data.find(c => c.slug === 'electronics')?.id
        },
        {
            name: 'Laptops',
            slug: 'laptops',
            description: 'Portable computers',
            parent_id: data.find(c => c.slug === 'electronics')?.id
        },
        
      // Virtual Goods subcategories
        {
            name: 'NFTs',
            slug: 'nfts',
            description: 'Non-fungible tokens',
            parent_id: data.find(c => c.slug === 'virtual-goods')?.id
        },
        {
            name: 'Digital Art',
            slug: 'digital-art',
            description: 'Artwork in digital format',
            parent_id: data.find(c => c.slug === 'virtual-goods')?.id
        }
    ];
    
    // Insert subcategories
    const { data: subCategoriesData, error: subCategoryError } = await supabase
    .from('categories')
    .insert(subCategories)
    .select();
    
    if (subCategoryError) {
        throw new Error(`Error inserting subcategories: ${subCategoryError.message}`);
    }
    
    // Return all categories (parents and children)
    const allCategories = [...data, ...subCategoriesData];
    return { success: true, data: allCategories };
    
    } catch (error) {
        console.error('Error seeding categories:', error);
        return { 
            success: false, 
            error: error instanceof Error ? error.message : 'Unknown error occurred while seeding categories'
        };
    }
};

/**
 * Seed sample products
 */
const seedProducts = async (categories: Category[]): Promise<{ success: boolean; error?: string }> => {
    try {
        // Check if products already exist
        const { data: existingProducts } = await supabase
            .from('products')
            .select('*');
        
        if (existingProducts && existingProducts.length > 0) {
            console.log('Products already exist, skipping seed.');
            return { success: true };
        }
    
        // Generate sample products
        const sampleProducts = generateSampleProducts(categories);
        
        // Insert products in batches to avoid request size limitations
        const batchSize = 10;
        for (let i = 0; i < sampleProducts.length; i += batchSize) {
            const batch = sampleProducts.slice(i, i + batchSize);
            const { error } = await supabase
            .from('products')
            .insert(batch);
            
            if (error) {
            throw new Error(`Error inserting products (batch ${i/batchSize + 1}): ${error.message}`);
            }
        }
    
        // Get the inserted products
        const { data: products, error: fetchError } = await supabase
            .from('products')
            .select('*');
        
        if (fetchError) {
            throw new Error(`Error fetching products after insert: ${fetchError.message}`);
        }
    
        // Seed product images
        await seedProductImages(products);
        
        return { success: true };
        } catch (error) {
            console.error('Error seeding products:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : 'Unknown error occurred while seeding products'
        };
    }
};

/**
 * Generate sample products for seeding
 */
const generateSampleProducts = (categories: Category[]): Omit<Product, 'id' | 'created_at' | 'updated_at' | 'rating' | 'rating_count'>[] => {
    const getCategoryId = (slug: string): string => {
    const category = categories.find(c => c.slug === slug);
    if (!category) throw new Error(`Category ${slug} not found`);
    return category.id;
    };
    
  // Generate sample seller ID (would be an actual user ID in a real application)
    const sellerId = '00000000-0000-0000-0000-000000000000';
    
    return [
    {
        title: 'StellarX Logo T-Shirt',
        description: 'Black cotton t-shirt with the StellarX logo printed on the front.',
        price: 25.99,
        category: getCategoryId('t-shirts'),
        seller_id: sellerId,
        stock: 50,
        slug: 'stellarx-logo-tshirt',
        featured: true,
    },
    {
        title: 'Stellar Network T-Shirt',
        description: 'Navy blue t-shirt featuring the Stellar network constellation design.',
        price: 29.99,
        category: getCategoryId('t-shirts'),
        seller_id: sellerId,
        stock: 35,
        slug: 'stellar-network-tshirt',
        featured: false,
    },
    
    // Hoodies
    {
        title: 'Crypto Enthusiast Hoodie',
        description: 'Warm hoodie with "Crypto Enthusiast" embroidered on the chest.',
        price: 49.99,
        category: getCategoryId('hoodies'),
        seller_id: sellerId,
        stock: 20,
        slug: 'crypto-enthusiast-hoodie',
        featured: true,
    },
    
    // Smartphones
    {
        title: 'CryptoPhone X1',
        description: 'Secure smartphone with built-in cryptocurrency wallet and enhanced security features.',
        price: 899.99,
        category: getCategoryId('smartphones'),
        seller_id: sellerId,
        stock: 10,
        slug: 'cryptophone-x1',
        featured: true,
    },
    
    // Laptops
    {
        title: 'DeveloperBook Pro',
        description: 'High-performance laptop optimized for blockchain development and testing.',
        price: 1499.99,
        category: getCategoryId('laptops'),
        seller_id: sellerId,
        stock: 5,
        slug: 'developerbook-pro',
        featured: false,
    },
    
    // Books
    {
        title: 'Understanding Stellar: A Beginner\'s Guide',
        description: 'Comprehensive guide to understanding the Stellar blockchain network and its ecosystem.',
        price: 24.99,
        category: getCategoryId('books'),
        seller_id: sellerId,
        stock: 100,
        slug: 'understanding-stellar-guide',
        featured: true,
    },
    
    // NFTs
    {
        title: 'Galactic Explorer NFT Collection',
        description: 'Limited edition NFT collection featuring space exploration artwork.',
        price: 199.99,
        category: getCategoryId('nfts'),
        seller_id: sellerId,
        stock: 10,
        slug: 'galactic-explorer-nft',
        featured: true,
    },
    
        // Digital Art
        {
        title: 'Abstract Constellation Digital Painting',
        description: 'High-resolution digital painting of an abstract stellar constellation.',
        price: 49.99,
        category: getCategoryId('digital-art'),
        seller_id: sellerId,
        stock: 999,
        slug: 'abstract-constellation-art',
        featured: false,
        },
    
    // Art
    {
        title: 'Handcrafted Stellar Mobile',
        description: 'Handmade hanging mobile featuring stellar and celestial elements.',
        price: 129.99,
        category: getCategoryId('art'),
        seller_id: sellerId,
        stock: 3,
        slug: 'handcrafted-stellar-mobile',
        featured: true,
    },
    
    // Collectibles
    {
        title: 'Limited Edition Stellar Foundation Coin',
        description: 'Physical commemorative coin celebrating the Stellar Foundation, numbered and authenticated.',
        price: 79.99,
        category: getCategoryId('collectibles'),
        seller_id: sellerId,
        stock: 15,
        slug: 'stellar-foundation-coin',
        featured: true,
    }
];
};

/**
 * Seed product images for the sample products
 */
const seedProductImages = async (products: Product[]): Promise<void> => {
    try {
    // Check if product images already exist
    const { data: existingImages } = await supabase
        .from('product_images')
        .select('*');
    
    if (existingImages && existingImages.length > 0) {
        console.log('Product images already exist, skipping seed.');
        return;
    }
    
    // In a real implementation, we would upload actual images to storage
    // For this demo, we'll create placeholder image records
    
    // Generate image records for each product
    const imageRecords = products.flatMap(product => {
      const imageCount = Math.floor(Math.random() * 3) + 1;
        
        return Array.from({ length: imageCount }, (_, i) => ({
            product_id: product.id,
            url: `https://via.placeholder.com/600x600?text=${encodeURIComponent(product.title)}`,
            alt_text: `${product.title} image ${i + 1}`,
            display_order: i,
            is_primary: i === 0 // First image is primary
        }));
    });
    
    // Insert image records in batches
    const batchSize = 10;
    for (let i = 0; i < imageRecords.length; i += batchSize) {
        const batch = imageRecords.slice(i, i + batchSize);
        const { error } = await supabase
        .from('product_images')
        .insert(batch);
        
        if (error) {
            throw new Error(`Error inserting product images (batch ${i/batchSize + 1}): ${error.message}`);
        }
    }
    
    console.log(`Seeded ${imageRecords.length} product images.`);
    } catch (error) {
        console.error('Error seeding product images:', error);
        throw error;
    }
};

// Run seeder (can be called from an admin interface or one-time setup script)
// seedDatabase().then(result => console.log('Database seed result:', result));