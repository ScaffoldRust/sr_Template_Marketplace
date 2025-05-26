import { supabase } from '../client';

/**
 * Applies Row-Level Security policies for the cart_items table
 */
export const applyCartPolicies = async (): Promise<{ success: boolean; error?: string }> => {
    try {
        await supabase.rpc('exec_sql', {
            sql: 'ALTER TABLE cart_items ENABLE ROW LEVEL SECURITY;'
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Users can view their own cart" 
            ON cart_items FOR SELECT USING (auth.uid() = user_id);`
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Users can add to their own cart" 
            ON cart_items FOR INSERT WITH CHECK (auth.uid() = user_id);`
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Users can update their own cart" 
            ON cart_items FOR UPDATE USING (auth.uid() = user_id);`
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Users can delete from their own cart" 
            ON cart_items FOR DELETE USING (auth.uid() = user_id);`
    });
    
    return { success: true };
    } catch (error) {
    console.error('Error applying cart RLS policies:', error);
    return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Unknown error applying cart policies'
    };
    }
};

/**
 * Checks if cart policies are correctly applied
 */
export const verifyCartPolicies = async (): Promise<{ 
    success: boolean; 
    missing?: string[];
    error?: string;
}> => {
    try {
        const { data, error } = await supabase.rpc('get_policies', {
            table_name: 'cart_items'
    });
    
    if (error) {
        throw new Error(`Error retrieving policies: ${error.message}`);
    }
    
    const requiredPolicies = [
        'Users can view their own cart',
        'Users can add to their own cart',
        'Users can update their own cart',
        'Users can delete from their own cart'
    ];
    
    type Policy = { policyname: string };
    const existingPolicies = data.map((policy: Policy) => policy.policyname);
    const missingPolicies = requiredPolicies.filter(policy => !existingPolicies.includes(policy));
    
    return { 
        success: missingPolicies.length === 0,
        missing: missingPolicies.length > 0 ? missingPolicies : undefined
    };
    } catch (error) {
    console.error('Error verifying cart RLS policies:', error);
    return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Unknown error verifying cart policies'
    };
    }
};