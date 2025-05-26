import 'dotenv/config';
import { Pool } from 'pg';
import fs from 'fs';
import path from 'path';
import { superAdmin } from '../lib/supabase/admin-client.js';
import { faker } from '@faker-js/faker';
import { fileURLToPath } from 'url';


const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const generateXlmPrice = () => parseFloat((Math.random() * 100 + 0.1).toFixed(2));

const pool = new Pool({
    connectionString: process.env.SUPABASE_POSTGRES_URL,
});

    async function runSqlFile(filePath: string) {
        try {
            const sql = await fs.promises.readFile(filePath, 'utf8');

            const client = await pool.connect();
            try {
                await client.query(sql);
                console.log(`✅ Executed ${path.basename(filePath)}`);
            } finally {
                client.release();
            }
        } catch (error) {
            console.error(`🔥 Failed to execute ${filePath}:`, error instanceof Error ? error.message : error);
            throw error;
        }
    }

    async function setupDatabase() {
    try {
        await superAdmin.rpc(`
            CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
            CREATE EXTENSION IF NOT EXISTS "pgcrypto";
            `);
        const schemaFiles = [
        'categories.sql',
        'products.sql',
        'product-images.sql',
        'cart-items.sql',
        'wishlist.sql',
        'rls-policies.sql',
        'index.sql'
        ];

        for (const file of schemaFiles) {
        await runSqlFile(path.join(__dirname, '../lib/db/schema', file));
        }

        const functionFiles = [
        'admin-check.sql',
        'stock-check.sql',
        'slugify.sql',
        'update-ratings.sql',
        'cart_search_functions.sql',
        'category_functions.sql'
        ];

        for (const file of functionFiles) {
        await runSqlFile(path.join(__dirname, '../lib/db/functions', file));
        }

        console.log('Database setup completed');
        process.exit(0);
    } catch (error) {
        console.error('Setup failed:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
    }

    async function seedDatabase() {
        try {
            const Categories = [
            { name: 'Clothing', slug: 'clothing', description: 'Apparel and wearable items' },
            { name: 'Electronics', slug: 'electronics', description: 'Electronic devices and accessories' },
            { name: 'Books', slug: 'books', description: 'Printed and digital books' },
            { name: 'Art', slug: 'art', description: 'Artwork and creative pieces' },
            { name: 'Virtual Goods', slug: 'virtual-goods', description: 'Digital items and virtual assets' },
            { name: 'Collectibles', slug: 'collectibles', description: 'Collectible items and memorabilia' }
            ];

            const { data: insertedCategories, error: categoryError } = await superAdmin
            .from('categories')
            .insert(Categories)
            .select();

            if (categoryError) throw categoryError;

            const subCategories = [
            { name: 'T-Shirts', slug: 't-shirts', description: 'Short-sleeved casual tops', parent_id: insertedCategories.find(c => c.slug === 'clothing')?.id },
            { name: 'Hoodies', slug: 'hoodies', description: 'Sweatshirts with hoods', parent_id: insertedCategories.find(c => c.slug === 'clothing')?.id },
            { name: 'Smartphones', slug: 'smartphones', description: 'Mobile phones and accessories', parent_id: insertedCategories.find(c => c.slug === 'electronics')?.id },
            { name: 'Laptops', slug: 'laptops', description: 'Portable computers', parent_id: insertedCategories.find(c => c.slug === 'electronics')?.id },
            { name: 'NFTs', slug: 'nfts', description: 'Non-fungible tokens', parent_id: insertedCategories.find(c => c.slug === 'virtual-goods')?.id },
            { name: 'Digital Art', slug: 'digital-art', description: 'Artwork in digital format', parent_id: insertedCategories.find(c => c.slug === 'virtual-goods')?.id }
            ];

            const { data: insertedSubCategories, error: subCategoryError } = await superAdmin
            .from('categories')
            .insert(subCategories)
            .select();

            if (subCategoryError) throw subCategoryError;

            const allCategories = [...insertedCategories, ...insertedSubCategories];
            const sellerId = await createTestSeller();
            const timestamp = Date.now();

            const products = Array.from({ length: 50 }, () => {
                const category = faker.helpers.arrayElement(allCategories);
                return {
                    title: faker.commerce.productName(),
                    description: faker.commerce.productDescription(),
                    price: generateXlmPrice(),
                    category: category.id,
                    rating: faker.number.float({ min: 1, max: 5, fractionDigits: 1 }),
                    rating_count: faker.number.int({ min: 0, max: 100 }),
                    seller_id: sellerId,
                    stock: faker.number.int({ min: 0, max: 100 }),
                    slug: `${faker.helpers.slugify(faker.commerce.productName())}-${timestamp}-${faker.string.uuid().substring(0, 8)}`,
                    featured: faker.datatype.boolean(),
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString()
                };
            });

            const { data: insertedProducts, error: productError } = await superAdmin
            .from('products')
            .insert(products)
            .select();

            if (productError) throw productError;

            const imageRecords = insertedProducts.flatMap(product => {
                const imageCount = Math.max(1, Math.floor(Math.random() * 4));
                return Array.from({ length: imageCount }, (_, i) => ({
                    product_id: product.id,
                    url: `https://via.placeholder.com/600x600?text=${encodeURIComponent(product.title)}`,
                    alt_text: `${product.title} - Image ${i + 1}`,
                    display_order: i,
                    is_primary: i === 0
                }));
            });

            const batchSize = 10;
            for (let i = 0; i < imageRecords.length; i += batchSize) {
                const batch = imageRecords.slice(i, i + batchSize);
                const { error } = await superAdmin.from('product_images').insert(batch);
                if (error) throw error;
            }

            console.log(`Seeded ${allCategories.length} categories`);
            console.log(`Seeded ${insertedProducts.length} products`);
            console.log(`Seeded ${imageRecords.length} product images`);
            process.exit(0);
            } catch (error) {
                console.error('Seeding failed:', error instanceof Error ? error.message : error);
                process.exit(1);
        }
    }

    async function createTestSeller() {
            const { data: { users }, error } = await superAdmin.auth.admin.listUsers();
            if (error) throw error;
            
            const existingSeller = users.find(u => u.email === 'test-seller@example.com');
            if (existingSeller) return existingSeller.id;
            
            const { data: { user }, error: createError } = await superAdmin.auth.admin.createUser({
                email: 'test-seller@example.com',
                password: 'testpassword123',
                email_confirm: true
            });
            
            if (createError) throw createError;
            return user!.id;
        }

    if (process.argv.includes('--setup')) {
    setupDatabase();
    } else {
    seedDatabase();
}

