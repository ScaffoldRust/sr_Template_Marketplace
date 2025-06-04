import { createClient } from '@/lib/supabase/client';
import type { Database, Store } from '@/lib/types/database.types';

type StoreInsert = Database['public']['Tables']['stores']['Insert'];
type StoreUpdate = Database['public']['Tables']['stores']['Update'];

const supabase = createClient();

export class StoresAPI {
  // Get store by ID
  static async getStore(storeId: string): Promise<Store | null> {
    const { data, error } = await supabase
      .from('stores')
      .select('*')
      .eq('id', storeId)
      .single();

    if (error) {
      console.error('Error fetching store:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Get stores by owner
  static async getStoresByOwner(ownerId: string): Promise<Store[]> {
    const { data, error } = await supabase
      .from('stores')
      .select('*')
      .eq('owner_id', ownerId)
      .order('created_at', { ascending: false });

    if (error) {
      console.error('Error fetching stores:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get store by slug
  static async getStoreBySlug(slug: string): Promise<Store | null> {
    const { data, error } = await supabase
      .from('stores')
      .select('*')
      .eq('slug', slug)
      .single();

    if (error) {
      console.error('Error fetching store by slug:', error);
      return null;
    }

    return data;
  }

  // Create new store
  static async createStore(store: StoreInsert): Promise<Store> {
    const { data, error } = await supabase
      .from('stores')
      .insert(store)
      .select()
      .single();

    if (error) {
      console.error('Error creating store:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Update store
  static async updateStore(storeId: string, updates: StoreUpdate): Promise<Store> {
    const { data, error } = await supabase
      .from('stores')
      .update(updates)
      .eq('id', storeId)
      .select()
      .single();

    if (error) {
      console.error('Error updating store:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Delete store
  static async deleteStore(storeId: string): Promise<void> {
    const { error } = await supabase
      .from('stores')
      .delete()
      .eq('id', storeId);

    if (error) {
      console.error('Error deleting store:', error);
      throw new Error(error.message);
    }
  }

  // Check if slug is available
  static async isSlugAvailable(slug: string, excludeStoreId?: string): Promise<boolean> {
    let query = supabase
      .from('stores')
      .select('id')
      .eq('slug', slug);

    if (excludeStoreId) {
      query = query.neq('id', excludeStoreId);
    }

    const { data, error } = await query;

    if (error) {
      console.error('Error checking slug availability:', error);
      throw new Error(error.message);
    }

    return data.length === 0;
  }

  // Search stores
  static async searchStores(
    query: string,
    limit = 10,
    offset = 0
  ): Promise<Store[]> {
    const { data, error } = await supabase
      .from('stores')
      .select('*')
      .or(`name.ilike.%${query}%, description.ilike.%${query}%`)
      .limit(limit)
      .range(offset, offset + limit - 1)
      .order('created_at', { ascending: false });

    if (error) {
      console.error('Error searching stores:', error);
      throw new Error(error.message);
    }

    return data || [];
  }
} 