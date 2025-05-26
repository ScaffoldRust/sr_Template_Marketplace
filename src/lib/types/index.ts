    export interface User {
        id: string;
        email: string;
        full_name?: string;
        is_admin?: boolean;
        created_at: string;
        updated_at: string;
    }
    
    export interface Product {
        id: string;
        title: string;
        description: string;
        price: number;
        category: string;
        rating: number;
        rating_count: number;
        seller_id: string;
        created_at: string;
        updated_at: string;
        stock: number;
        slug: string;
        featured: boolean;
    };
    
    export interface Category {
        id: string;
        name: string;
        slug: string;
        description: string;
        parent_id: string | null;
    };
    
    export interface ProductImage {
        id: string;
        product_id: string;
        url: string;
        alt_text: string;
        display_order: number;
        is_primary: boolean;
    };
    
    export interface WishlistItem {
        id: string;
        user_id: string;
        product_id: string;
        created_at: string;
    };
    
    export interface CartItem {
        id: string;
        user_id: string;
        product_id: string;
        quantity: number;
        created_at: string;
        updated_at: string;
    }
    
    export interface ProductRating {
        id: string;
        product_id: string;
        user_id: string;
        rating: number; // 1-5
        comment?: string;
        created_at: string;
    }
    
    export enum UserRole {
        USER = 'user',
        ADMIN = 'admin',
        SELLER = 'seller'
    }
    
    // Types for seeding data
    export interface ProductSeed {
        title: string;
        description: string;
        price: number;
        category: string;
        stock: number;
        featured: boolean;
        images: {
        url: string;
        alt_text: string;
        is_primary: boolean;
        }[];
    }
    
    export interface CategorySeed {
        name: string;
        description: string;
        parent?: string;
    }

    
    
  // Export API types
  export * from './api.types';