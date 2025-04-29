import { applyCartPolicies, verifyCartPolicies } from './cart';
import { applyCategoriesPolicies, verifyCategoriesPolicies } from './categories';
import { applyProductsPolicies, verifyProductsPolicies } from './products';
import { applyWishlistPolicies, verifyWishlistPolicies } from './wishlist';

/**
 * Apply all RLS policies for the marketplace
 */
export const applyAllPolicies = async (): Promise<{
    success: boolean;
    results: {
        products: { success: boolean; error?: string };
        categories: { success: boolean; error?: string };
        wishlist: { success: boolean; error?: string };
        cart: { success: boolean; error?: string };
    };
    }> => {
        const productsResult = await applyProductsPolicies();
        const categoriesResult = await applyCategoriesPolicies();
        const wishlistResult = await applyWishlistPolicies();
        const cartResult = await applyCartPolicies();
    
        const allSuccessful =
        productsResult.success && 
        categoriesResult.success && 
        wishlistResult.success && 
        cartResult.success;
    
    return {
        success: allSuccessful,
        results: {
        products: productsResult,
        categories: categoriesResult,
        wishlist: wishlistResult,
        cart: cartResult
        }
    };
    };

    /**
     * Verify all RLS policies are applied correctly
     */
    export const verifyAllPolicies = async (): Promise<{
    success: boolean;
    results: {
        products: { success: boolean; missing?: string[]; error?: string };
        categories: { success: boolean; missing?: string[]; error?: string };
        wishlist: { success: boolean; missing?: string[]; error?: string };
        cart: { success: boolean; missing?: string[]; error?: string };
    };
    }> => {
        const productsResult = await verifyProductsPolicies();
        const categoriesResult = await verifyCategoriesPolicies();
        const wishlistResult = await verifyWishlistPolicies();
        const cartResult = await verifyCartPolicies();
    
        const allSuccessful = 
        productsResult.success && 
        categoriesResult.success && 
        wishlistResult.success && 
        cartResult.success;
    
    return {
        success: allSuccessful,
        results: {
        products: productsResult,
        categories: categoriesResult,
        wishlist: wishlistResult,
        cart: cartResult
        }
    };
    };

    export {
        applyCartPolicies,
        verifyCartPolicies,
        applyCategoriesPolicies,
        verifyCategoriesPolicies,
        applyProductsPolicies,
        verifyProductsPolicies,
        applyWishlistPolicies,
        verifyWishlistPolicies
};