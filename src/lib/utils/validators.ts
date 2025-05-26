import { CartAddRequest, ProductSearchParams, RatingAddRequest } from '../types';
import { Product } from '../types/database.types';
import { Category } from '../types/database.types';

/**
 * Validate product data before insertion or update
 * @param product The product data to validate
 * @returns An error message if validation fails, null otherwise
 */
export function validateProduct(product: Product): string | null {
    if (!product.title || product.title.trim() === '') {
        return 'Product title is required';
    }

    if (!product.description || product.description.trim() === '') {
        return 'Product description is required';
    }

    if (typeof product.price !== 'number' || product.price <= 0) {
        return 'Product price must be a positive number';
    }

    if (typeof product.stock !== 'number' || product.stock < 0) {
        return 'Product stock must be a non-negative number';
    }

    if (!product.category) {
        return 'Product category is required';
    }

    return null;
}

/**
 * Validate category data before insertion or update
 * @param category The category data to validate
 * @returns An error message if validation fails, null otherwise
 */
export function validateCategory(category: Category): string | null {
if (!category.name || category.name.trim() === '') {
    return 'Category name is required';
}

if (!category.description || category.description.trim() === '') {
    return 'Category description is required';
}

    return null;
}

/**
 * Validate product search params
 * @param params Search parameters to validate
 * @returns An error message if validation fails, null otherwise
 */
export function validateSearchParams(params: ProductSearchParams): string | null {
    if (params.minPrice !== undefined && (isNaN(params.minPrice) || params.minPrice < 0)) {
        return 'Minimum price must be a non-negative number';
    }

    if (params.maxPrice !== undefined && (isNaN(params.maxPrice) || params.maxPrice < 0)) {
        return 'Maximum price must be a non-negative number';
    }

    if (
        params.minPrice !== undefined &&
        params.maxPrice !== undefined &&
        params.minPrice > params.maxPrice
    ) {
        return 'Minimum price cannot be greater than maximum price';
    }

    if (
        params.minRating !== undefined &&
        (isNaN(params.minRating) || params.minRating < 1 || params.minRating > 5)
    ) {
        return 'Minimum rating must be between 1 and 5';
    }

    if (params.page !== undefined && (isNaN(params.page) || params.page < 1)) {
        return 'Page must be a positive number';
    }

    if (params.limit !== undefined && (isNaN(params.limit) || params.limit < 1)) {
        return 'Limit must be a positive number';
    }

    return null;
}

/**
 * Validate cart add request
 * @param request The cart add request to validate
 * @returns An error message if validation fails, null otherwise
 */
export function validateCartAddRequest(request: CartAddRequest): string | null {
    if (!request.productId) {
        return 'Product ID is required';
    }

    if (typeof request.quantity !== 'number' || request.quantity < 1) {
        return 'Quantity must be a positive number';
    }

    return null;
}

/**
 * Validate rating add request
 * @param request The rating add request to validate
 * @returns An error message if validation fails, null otherwise
 */
export function validateRatingAddRequest(request: RatingAddRequest): string | null {
    if (!request.productId) {
        return 'Product ID is required';
    }

    if (
        typeof request.rating !== 'number' ||
        request.rating < 1 ||
        request.rating > 5 ||
        !Number.isInteger(request.rating)
    ) {
        return 'Rating must be an integer between 1 and 5';
    }

    return null;
}

/**
 * Validate UUID format
 * @param id The ID to validate
 * @returns True if the ID is a valid UUID, false otherwise
 */
export function isValidUuid(id: string): boolean {
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    return uuidRegex.test(id);
}

/**
 * Validate pagination parameters and return default values if not provided
 * @param page The page number
 * @param limit The number of items per page
 * @returns An object with valid page and limit values
 */
export function validatePagination(page?: number, limit?: number): { page: number; limit: number } {
    const validPage = page && page > 0 ? Math.floor(page) : 1;
    const validLimit = limit && limit > 0 ? Math.min(Math.floor(limit), 100) : 20;

    return { page: validPage, limit: validLimit };
}

/**
 * Check if a string is valid for use in a slug
 * @param str The string to check
 * @returns True if the string is valid for use in a slug, false otherwise
 */
export function isValidSlugString(str: string): boolean {
    return /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(str);
}