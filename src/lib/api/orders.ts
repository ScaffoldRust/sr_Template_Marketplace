import { createClient } from '@/lib/supabase/client';
import type { Database, Order, OrderItem, SellerRecentOrdersView } from '@/lib/types/database.types';

type OrderInsert = Database['public']['Tables']['orders']['Insert'];
type OrderUpdate = Database['public']['Tables']['orders']['Update'];
type OrderItemInsert = Database['public']['Tables']['order_items']['Insert'];

const supabase = createClient();

export interface OrderFilters {
  status?: string[];
  startDate?: string;
  endDate?: string;
  customerId?: string;
  minAmount?: number;
  maxAmount?: number;
}

export interface OrdersPagination {
  page?: number;
  pageSize?: number;
}

export class OrdersAPI {
  // Get order by ID
  static async getOrder(orderId: string): Promise<Order | null> {
    const { data, error } = await supabase
      .from('orders')
      .select('*')
      .eq('id', orderId)
      .single();

    if (error) {
      console.error('Error fetching order:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Get orders for a store with filters and pagination
  static async getStoreOrders(
    storeId: string,
    filters: OrderFilters = {},
    pagination: OrdersPagination = {}
  ): Promise<{ orders: Order[]; totalCount: number }> {
    const { page = 1, pageSize = 10 } = pagination;
    const offset = (page - 1) * pageSize;

    let query = supabase
      .from('orders')
      .select('*', { count: 'exact' })
      .eq('store_id', storeId);

    // Apply filters
    if (filters.status && filters.status.length > 0) {
      query = query.in('status', filters.status);
    }

    if (filters.startDate) {
      query = query.gte('created_at', filters.startDate);
    }

    if (filters.endDate) {
      query = query.lte('created_at', filters.endDate);
    }

    if (filters.customerId) {
      query = query.eq('user_id', filters.customerId);
    }

    if (filters.minAmount) {
      query = query.gte('total_amount', filters.minAmount);
    }

    if (filters.maxAmount) {
      query = query.lte('total_amount', filters.maxAmount);
    }

    const { data, error, count } = await query
      .order('created_at', { ascending: false })
      .range(offset, offset + pageSize - 1);

    if (error) {
      console.error('Error fetching store orders:', error);
      throw new Error(error.message);
    }

    return {
      orders: data || [],
      totalCount: count || 0,
    };
  }

  // Get recent orders view for seller dashboard
  static async getRecentOrdersView(
    storeId: string,
    limit = 10
  ): Promise<SellerRecentOrdersView[]> {
    const { data, error } = await supabase
      .from('seller_recent_orders_view')
      .select('*')
      .eq('store_id', storeId)
      .order('created_at', { ascending: false })
      .limit(limit);

    if (error) {
      console.error('Error fetching recent orders view:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get order items for an order
  static async getOrderItems(orderId: string): Promise<OrderItem[]> {
    const { data, error } = await supabase
      .from('order_items')
      .select(`
        *,
        products (
          title,
          name,
          variant,
          price
        )
      `)
      .eq('order_id', orderId);

    if (error) {
      console.error('Error fetching order items:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Create new order
  static async createOrder(order: OrderInsert): Promise<Order> {
    const { data, error } = await supabase
      .from('orders')
      .insert(order)
      .select()
      .single();

    if (error) {
      console.error('Error creating order:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Update order
  static async updateOrder(orderId: string, updates: OrderUpdate): Promise<Order> {
    const { data, error } = await supabase
      .from('orders')
      .update(updates)
      .eq('id', orderId)
      .select()
      .single();

    if (error) {
      console.error('Error updating order:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Update order status
  static async updateOrderStatus(
    orderId: string,
    status: Order['status'],
    notes?: string
  ): Promise<Order> {
    const { data, error } = await supabase
      .from('orders')
      .update({ 
        status,
        updated_at: new Date().toISOString(),
      })
      .eq('id', orderId)
      .select()
      .single();

    if (error) {
      console.error('Error updating order status:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Add order items
  static async addOrderItems(orderItems: OrderItemInsert[]): Promise<OrderItem[]> {
    const { data, error } = await supabase
      .from('order_items')
      .insert(orderItems)
      .select();

    if (error) {
      console.error('Error adding order items:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get order status history
  static async getOrderStatusHistory(orderId: string) {
    const { data, error } = await supabase
      .from('order_status_history')
      .select('*')
      .eq('order_id', orderId)
      .order('changed_at', { ascending: false });

    if (error) {
      console.error('Error fetching order status history:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get order analytics for store
  static async getOrderAnalytics(
    storeId: string,
    startDate: string,
    endDate: string
  ) {
    const { data, error } = await supabase
      .from('orders')
      .select('status, total_amount, created_at')
      .eq('store_id', storeId)
      .gte('created_at', startDate)
      .lte('created_at', endDate);

    if (error) {
      console.error('Error fetching order analytics:', error);
      throw new Error(error.message);
    }

    // Process analytics data
    const analytics = {
      totalOrders: data?.length || 0,
      totalRevenue: data?.reduce((sum, order) => sum + order.total_amount, 0) || 0,
      ordersByStatus: data?.reduce((acc, order) => {
        acc[order.status] = (acc[order.status] || 0) + 1;
        return acc;
      }, {} as Record<string, number>) || {},
      averageOrderValue: data?.length ? 
        (data.reduce((sum, order) => sum + order.total_amount, 0) / data.length) : 0,
    };

    return analytics;
  }

  // Cancel order
  static async cancelOrder(orderId: string, reason?: string): Promise<Order> {
    return this.updateOrderStatus(orderId, 'cancelled', reason);
  }

  // Refund order
  static async refundOrder(orderId: string, reason?: string): Promise<Order> {
    return this.updateOrderStatus(orderId, 'refunded', reason);
  }
} 