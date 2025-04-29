import { supabase } from '../client';

/**
 * Applies Row-Level Security policies for the products table
 */
export const applyProductsPolicies = async (): Promise<{ success: boolean; error?: string }> => {
    try {
            await supabase.rpc('exec_sql', {
            sql: 'ALTER TABLE products ENABLE ROW LEVEL SECURITY;'
        });
        
        await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Anyone can view products" 
                ON products FOR SELECT USING (true);`
        });
        
        await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Sellers can insert products" 
                ON products FOR INSERT WITH CHECK (auth.uid() = seller_id);`
        });
        
        await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Sellers can update their products" 
                ON products FOR UPDATE USING (auth.uid() = seller_id);`
        });
        
        await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Sellers can delete their products" 
                ON products FOR DELETE USING (auth.uid() = seller_id);`
        });
        
        return { success: true };
    } catch (error) {
            console.error('Error applying products RLS policies:', error);
            return { 
                success: false, 
                error: error instanceof Error ? error.message : 'Unknown error applying products policies'
        };
    }
    };

    /**
     * Checks if products policies are correctly applied
     */
    export const verifyProductsPolicies = async (): Promise<{
        success: boolean; 
        missing?: string[];
        error?: string;
    }> => {
    try {
            const { data, error } = await supabase.rpc('get_policies', {
            table_name: 'products'
        });
        
        if (error) {
            throw new Error(`Error retrieving policies: ${error.message}`);
        }
        
        const requiredPolicies = [
            'Anyone can view products',
            'Sellers can insert products',
            'Sellers can update their products',
            'Sellers can delete their products'
        ];
        
        type Policy = { policyname: string };
        const existingPolicies = data.map((policy: Policy) => policy.policyname);
        const missingPolicies = requiredPolicies.filter(policy => !existingPolicies.includes(policy));
        
        return { 
            success: missingPolicies.length === 0,
            missing: missingPolicies.length > 0 ? missingPolicies : undefined
        };
    } catch (error) {
            console.error('Error verifying products RLS policies:', error);
        return { 
            success: false, 
            error: error instanceof Error ? error.message : 'Unknown error verifying products policies'
        };
    }
};