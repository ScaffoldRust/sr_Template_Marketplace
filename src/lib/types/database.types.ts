export type Database = {
    public: {
        Tables: {
            products: {
                Row: Product;
                Insert: Omit<Product, 'id' | 'created_at' | 'updated_at' | 'rating' | 'rating_count'>;
                Update: Partial<Omit<Product, 'id' | 'created_at' | 'updated_at'>>;
            };
            categories: {
                Row: Category;
                Insert: Omit<Category, 'id'>;
                Update: Partial<Omit<Category, 'id'>>;
            };
            product_images: {
                Row: ProductImage;
                Insert: Omit<ProductImage, 'id'>;
                Update: Partial<Omit<ProductImage, 'id'>>;
            };
            wishlist_items: {
                Row: WishlistItem;
                Insert: Omit<WishlistItem, 'id' | 'created_at'>;
                Update: Partial<Omit<WishlistItem, 'id' | 'created_at'>>;
            };
            cart_items: {
                Row: CartItem;
                Insert: Omit<CartItem, 'id' | 'created_at' | 'updated_at'>;
                Update: Partial<Omit<CartItem, 'id' | 'created_at' | 'updated_at'>>;
            };
        };
        Functions: {
                search_products: {
            Args: {
                search_query: string;
                category_id?: string;
                min_price?: number;
                max_price?: number;
                min_rating?: number;
                limit?: number;
                offset?: number;
            };
            Returns: Product[];
            };
            calculate_cart_total: {
            Args: {
                user_id: string;
            };
            Returns: {
                total: number;
            };
            };
        };
        };
    };
    
    export type Product = {
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
    
    export type Category = {
        id: string;
        name: string;
        slug: string;
        description: string;
        parent_id: string | null;
    };
    
    export type ProductImage = {
        id: string;
        product_id: string;
        url: string;
        alt_text: string;
        display_order: number;
        is_primary: boolean;
    };
    
    export type WishlistItem = {
        id: string;
        user_id: string;
        product_id: string;
        created_at: string;
    };
    
    export type CartItem = {
        id: string;
        user_id: string;
        product_id: string;
        quantity: number;
        created_at: string;
        updated_at: string;
    };
    
    // Pagination and filter types
    export type PaginationParams = {
        page?: number;
        pageSize?: number;
    };
    
    export type ProductFilter = {
        category?: string;
        minPrice?: number;
        maxPrice?: number;
        minRating?: number;
        featured?: boolean;
        sortBy?: 'price' | 'rating' | 'created_at';
        sortOrder?: 'asc' | 'desc';
    };