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
            stores: {
                Row: Store;
                Insert: Omit<Store, 'id' | 'created_at' | 'updated_at' | 'last_updated' | 'revenue_total' | 'active_product_count' | 'pending_order_count' | 'average_rating' | 'rating_count'>;
                Update: Partial<Omit<Store, 'id' | 'created_at' | 'updated_at' | 'last_updated'>>;
            };
            orders: {
                Row: Order;
                Insert: Omit<Order, 'id' | 'created_at' | 'updated_at'>;
                Update: Partial<Omit<Order, 'id' | 'created_at' | 'updated_at'>>;
            };
            order_items: {
                Row: OrderItem;
                Insert: Omit<OrderItem, 'id' | 'created_at'>;
                Update: Partial<Omit<OrderItem, 'id' | 'created_at'>>;
            };
            store_analytics: {
                Row: StoreAnalytics;
                Insert: Omit<StoreAnalytics, 'id' | 'created_at' | 'updated_at'>;
                Update: Partial<Omit<StoreAnalytics, 'id' | 'created_at' | 'updated_at'>>;
            };
            store_performance_goals: {
                Row: StorePerformanceGoal;
                Insert: Omit<StorePerformanceGoal, 'id' | 'created_at' | 'updated_at'>;
                Update: Partial<Omit<StorePerformanceGoal, 'id' | 'created_at' | 'updated_at'>>;
            };
            order_status_history: {
                Row: OrderStatusHistory;
                Insert: Omit<OrderStatusHistory, 'id' | 'created_at'>;
                Update: Partial<Omit<OrderStatusHistory, 'id' | 'created_at'>>;
            };
            reviews: {
                Row: Review;
                Insert: Omit<Review, 'id' | 'created_at' | 'updated_at'>;
                Update: Partial<Omit<Review, 'id' | 'created_at' | 'updated_at'>>;
            };
        };
        Views: {
            seller_dashboard_view: {
                Row: SellerDashboardView;
            };
            seller_recent_orders_view: {
                Row: SellerRecentOrdersView;
            };
            store_performance_view: {
                Row: StorePerformanceView;
            };
            store_analytics_summary_view: {
                Row: StoreAnalyticsSummaryView;
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
            calculate_store_revenue: {
                Args: {
                    store_id: string;
                };
                Returns: number;
            };
            calculate_performance_percentage: {
                Args: {
                    current_value: number;
                    target_value: number;
                };
                Returns: number;
            };
            get_store_analytics: {
                Args: {
                    store_id: string;
                    start_date: string;
                    end_date: string;
                };
                Returns: StoreAnalytics[];
            };
            update_goal_progress: {
                Args: {
                    goal_id: string;
                    new_current_value: number;
                };
                Returns: boolean;
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

export type Store = {
    id: string;
    name: string;
    description: string | null;
    owner_id: string;
    slug: string;
    logo_url: string | null;
    banner_url: string | null;
    contact_email: string | null;
    phone: string | null;
    address: string | null;
    city: string | null;
    state: string | null;
    country: string | null;
    postal_code: string | null;
    website_url: string | null;
    social_media: Record<string, any>;
    revenue_total: number;
    active_product_count: number;
    pending_order_count: number;
    average_rating: number;
    rating_count: number;
    monthly_sales_goal: number | null;
    customer_goal: number | null;
    review_goal: number | null;
    last_updated: string;
    created_at: string;
    updated_at: string;
};

export type Order = {
    id: string;
    user_id: string;
    store_id: string;
    status: 'created' | 'processing' | 'paid' | 'shipped' | 'delivered' | 'cancelled' | 'refunded';
    total_amount: number;
    subtotal: number;
    tax_amount: number;
    shipping_amount: number;
    discount_amount: number;
    currency: string;
    payment_method: string | null;
    payment_id: string | null;
    shipping_address: Record<string, any> | null;
    billing_address: Record<string, any> | null;
    tracking_number: string | null;
    notes: string | null;
    created_at: string;
    updated_at: string;
};

export type OrderItem = {
    id: string;
    order_id: string;
    product_id: string;
    quantity: number;
    price_at_purchase: number;
    total_price: number;
    created_at: string;
};

export type StoreAnalytics = {
    id: string;
    store_id: string;
    date: string;
    revenue: number;
    order_count: number;
    new_customers: number;
    returning_customers: number;
    average_order_value: number;
    conversion_rate: number;
    view_count: number;
    created_at: string;
    updated_at: string;
};

export type StorePerformanceGoal = {
    id: string;
    store_id: string;
    goal_type: 'sales' | 'customers' | 'reviews' | 'conversion' | 'aov';
    target_value: number;
    current_value: number;
    time_period: 'daily' | 'weekly' | 'monthly' | 'quarterly' | 'yearly';
    start_date: string;
    end_date: string;
    created_at: string;
    updated_at: string;
};

export type OrderStatusHistory = {
    id: string;
    order_id: string;
    status: 'created' | 'processing' | 'paid' | 'shipped' | 'delivered' | 'cancelled' | 'refunded';
    changed_at: string;
    changed_by: string | null;
    notes: string | null;
    created_at: string;
};

export type Review = {
    id: string;
    product_id: string;
    store_id: string;
    user_id: string;
    rating: number;
    comment: string | null;
    created_at: string;
    updated_at: string;
};

export type SellerDashboardView = {
    store_id: string;
    store_name: string;
    owner_id: string;
    total_revenue: number;
    active_products: number;
    pending_orders: number;
    average_rating: number;
    rating_count: number;
    total_orders: number;
    monthly_sales_goal: number | null;
    customer_goal: number | null;
    review_goal: number | null;
    last_updated: string;
};

export type SellerRecentOrdersView = {
    order_id: string;
    store_id: string;
    total_amount: number;
    status: string;
    created_at: string;
    updated_at: string;
    tracking_number: string | null;
    customer_name: string | null;
    customer_email: string;
    product_id: string;
    quantity: number;
    price_at_purchase: number;
    total_price: number;
    product_name: string;
    product_variant: string | null;
    store_name: string;
};

export type StorePerformanceView = {
    store_id: string;
    store_name: string;
    goal_id: string;
    goal_type: string;
    target_value: number;
    current_value: number;
    percentage: number;
    time_period: string;
    start_date: string;
    end_date: string;
    created_at: string;
    updated_at: string;
};

export type StoreAnalyticsSummaryView = {
    store_id: string;
    store_name: string;
    date: string;
    revenue: number;
    order_count: number;
    new_customers: number;
    returning_customers: number;
    average_order_value: number;
    conversion_rate: number;
    view_count: number;
    revenue_week_ago: number | null;
    revenue_month_ago: number | null;
    orders_week_ago: number | null;
    orders_month_ago: number | null;
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