import { supabase } from '../client';

/**
 * Applies Row-Level Security policies for the categories table
 */
export const applyCategoriesPolicies = async (): Promise<{ success: boolean; error?: string }> => {
    try {
    await supabase.rpc('exec_sql', {
        sql: 'ALTER TABLE categories ENABLE ROW LEVEL SECURITY;'
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Anyone can view categories" 
            ON categories FOR SELECT USING (true);`
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Only admins can insert categories" 
            ON categories FOR INSERT WITH CHECK (is_admin());`
    });
    
    await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Only admins can update categories" 
                ON categories FOR UPDATE USING (is_admin());`
    });
    
        await supabase.rpc('exec_sql', {
        sql: `CREATE POLICY "Only admins can delete categories" 
                ON categories FOR DELETE USING (is_admin());`
        });
        
        return { success: true };
    } catch (error) {
            console.error('Error applying categories RLS policies:', error);
            return { 
            success: false, 
            error: error instanceof Error ? error.message : 'Unknown error applying categories policies'
        };
    }
};

/**
 * Checks if categories policies are correctly applied
 */
export const verifyCategoriesPolicies = async (): Promise<{ 
    success: boolean; 
    missing?: string[];
    error?: string;
}> => {
    try {
    const { data, error } = await supabase.rpc('get_policies', {
        table_name: 'categories'
    });
    
    if (error) {
        throw new Error(`Error retrieving policies: ${error.message}`);
    }
    
    const requiredPolicies = [
        'Anyone can view categories',
        'Only admins can insert categories',
        'Only admins can update categories',
        'Only admins can delete categories'
    ];
    
    type Policy = { policyname: string };
    const existingPolicies = data.map((policy: Policy) => policy.policyname);
    const missingPolicies = requiredPolicies.filter(policy => !existingPolicies.includes(policy));
    
    return { 
        success: missingPolicies.length === 0,
        missing: missingPolicies.length > 0 ? missingPolicies : undefined
    };
    } catch (error) {
        console.error('Error verifying categories RLS policies:', error);
            return {
                success: false, 
                error: error instanceof Error ? error.message : 'Unknown error verifying categories policies'
        };
    }
};