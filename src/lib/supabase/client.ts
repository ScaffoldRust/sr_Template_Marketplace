import { createClient } from '@supabase/supabase-js';
import { Database } from '../types/database.types';

// Environment variables should be properly typed
const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL as string;
const supabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY as string;
const supabaseServiceRoleKey = process.env.SUPABASE_SERVICE_ROLE_KEY as string;

// Validate environment variables are present
if (!supabaseUrl || !supabaseAnonKey) {
    throw new Error('Missing Supabase environment variables. Please check your .env file.');
}

// Create a regular client for public use
const supabase = createClient<Database>(supabaseUrl, supabaseAnonKey);

// Create an admin client with service role key for bypassing RLS
// This should only be used in server environments
const adminClient = createClient<Database>(
    supabaseUrl,
    supabaseServiceRoleKey,
    {
        auth: {
        persistSession: false,
        autoRefreshToken: false,
        },
    }
);

export { supabase, adminClient };