import { Product } from './database.types';
import { Category } from './database.types';
import { WishlistItem } from './database.types';
import { CartItem } from './database.types';
    export interface PaginationParams {
        page?: number;
        limit?: number;
    }
    
  // Common sorting parameters
    export interface SortParams {
        sortBy?: string;
        sortDirection?: 'asc' | 'desc';
    }
    
  // Product filter parameters
    export interface ProductFilterParams {
        category?: string;
        minPrice?: number;
        maxPrice?: number;
        minRating?: number;
        featured?: boolean;
    }
    
    // Product search parameters
    export interface ProductSearchParams extends PaginationParams, SortParams, ProductFilterParams {
        query?: string;
    }
    
  // Product API responses
    export interface ProductListResponse {
        data: Product[];
        total: number;
        page: number;
        limit: number;
        totalPages: number;
    }
    
    export interface ProductResponse {
        data: Product | null;
        error?: string;
    }
    
  // Category API responses
    export interface CategoryListResponse {
        data: Category[];
        total: number;
    }
    
    export interface CategoryResponse {
        data: Category | null;
        error?: string;
    }
    
  // Wishlist API requests and responses
    export interface WishlistAddRequest {
        productId: string;
    }
    
    export interface WishlistResponse {
        data: WishlistItem[];
        total: number;
    }
    
  // Cart API requests and responses
    export interface CartAddRequest {
        productId: string;
        quantity: number;
    }
    
    export interface CartUpdateRequest {
        quantity: number;
    }
    
    export interface CartResponse {
        items: CartItem[];
        total: number;
        itemCount: number;
    }
    
  // Search API responses
    export interface SearchResponse {
        products: Product[];
        total: number;
        page: number;
        limit: number;
        totalPages: number;
    }
    
  // Rating API requests
    export interface RatingAddRequest {
        productId: string;
        rating: number;
        comment?: string;
    }
    
  // API Error response
    export interface ApiErrorResponse {
        error: string;
        code?: string;
        details?: unknown;
    }