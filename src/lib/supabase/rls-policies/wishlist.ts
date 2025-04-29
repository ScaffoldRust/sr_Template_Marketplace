import { supabase } from '../client';

/**
 * Applies Row-Level Security policies for the wishlist_items table
 */
export const applyWishlistPolicies = async (): Promise<{ success: boolean; error?: string }> => {
    try {
            await supabase.rpc('exec_sql', {
            sql: 'ALTER TABLE wishlist_items ENABLE ROW LEVEL SECURITY;'
        });
        
        await supabase.rpc('exec_sql', {
            sql: `CREATE POLICY "Users can view their own wishlist" 
                    ON wishlist_items FOR SELECT USING (auth.uid() = user_id);`
        });
        
        await supabase.rpc('exec_sql', {
            sql: `CREATE POLICY "Users can add to their own wishlist" 
                    ON wishlist_items FOR INSERT WITH CHECK (auth.uid() = user_id);`
        });
        
        await supabase.rpc('exec_sql', {
            sql: `CREATE POLICY "Users can remove from their own wishlist" 
                    ON wishlist_items FOR DELETE USING (auth.uid() = user_id);`
        });
        
        return { success: true };
    } catch (error) {
            console.error('Error applying wishlist RLS policies:', error);
        return { 
            success: false, 
            error: error instanceof Error ? error.message : 'Unknown error applying wishlist policies'
        };
    }
    };

    /**
     * Checks if wishlist policies are correctly applied
     */
    export const verifyWishlistPolicies = async (): Promise<{ 
        success: boolean; 
        missing?: string[];
        error?: string;
    }> => {
    try {
            const { data, error } = await supabase.rpc('get_policies', {
            table_name: 'wishlist_items'
        });
        
        if (error) {
            throw new Error(`Error retrieving policies: ${error.message}`);
        }
        
        const requiredPolicies = [
            'Users can view their own wishlist',
            'Users can add to their own wishlist',
            'Users can remove from their own wishlist'
        ];
        
        type policy ={policyname: string};
        const existingPolicies = data.map((policy: policy) => policy.policyname);
        const missingPolicies = requiredPolicies.filter(policy => !existingPolicies.includes(policy));
        
        return { 
            success: missingPolicies.length === 0,
            missing: missingPolicies.length > 0 ? missingPolicies : undefined
        };
    } catch (error) {
            console.error('Error verifying wishlist RLS policies:', error);
        return { 
            success: false, 
            error: error instanceof Error ? error.message : 'Unknown error verifying wishlist policies'
        };
    }
};