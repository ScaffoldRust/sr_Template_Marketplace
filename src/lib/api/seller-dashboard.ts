import { createClient } from '@/lib/supabase/client';
import type { 
  SellerDashboardView,
  StoreAnalytics,
  StorePerformanceGoal,
  StorePerformanceView,
  StoreAnalyticsSummaryView,
  Database
} from '@/lib/types/database.types';

const supabase = createClient();

export interface DashboardFilters {
  startDate?: string;
  endDate?: string;
  period?: 'day' | 'week' | 'month' | 'quarter' | 'year';
}

export interface PerformanceGoalInput {
  goal_type: 'sales' | 'customers' | 'reviews' | 'conversion' | 'aov';
  target_value: number;
  time_period: 'daily' | 'weekly' | 'monthly' | 'quarterly' | 'yearly';
  start_date: string;
  end_date: string;
}

export class SellerDashboardAPI {
  // Get dashboard overview for a store
  static async getDashboardOverview(storeId: string): Promise<SellerDashboardView | null> {
    const { data, error } = await supabase
      .from('seller_dashboard_view')
      .select('*')
      .eq('store_id', storeId)
      .single();

    if (error) {
      console.error('Error fetching dashboard overview:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Get store analytics for date range
  static async getStoreAnalytics(
    storeId: string,
    startDate: string,
    endDate: string
  ): Promise<StoreAnalytics[]> {
    const { data, error } = await supabase
      .rpc('get_store_analytics', {
        store_id: storeId,
        start_date: startDate,
        end_date: endDate
      });

    if (error) {
      console.error('Error fetching store analytics:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get analytics summary view with comparisons
  static async getAnalyticsSummary(
    storeId: string,
    filters: DashboardFilters = {}
  ): Promise<StoreAnalyticsSummaryView[]> {
    let query = supabase
      .from('store_analytics_summary_view')
      .select('*')
      .eq('store_id', storeId);

    if (filters.startDate) {
      query = query.gte('date', filters.startDate);
    }

    if (filters.endDate) {
      query = query.lte('date', filters.endDate);
    }

    const { data, error } = await query
      .order('date', { ascending: false })
      .limit(30); // Last 30 days by default

    if (error) {
      console.error('Error fetching analytics summary:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get performance goals for store
  static async getPerformanceGoals(storeId: string): Promise<StorePerformanceView[]> {
    const { data, error } = await supabase
      .from('store_performance_view')
      .select('*')
      .eq('store_id', storeId)
      .order('created_at', { ascending: false });

    if (error) {
      console.error('Error fetching performance goals:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Create new performance goal
  static async createPerformanceGoal(
    storeId: string,
    goal: PerformanceGoalInput
  ): Promise<StorePerformanceGoal> {
    const { data, error } = await supabase
      .from('store_performance_goals')
      .insert({
        store_id: storeId,
        ...goal
      })
      .select()
      .single();

    if (error) {
      console.error('Error creating performance goal:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Update performance goal
  static async updatePerformanceGoal(
    goalId: string,
    updates: Partial<PerformanceGoalInput>
  ): Promise<StorePerformanceGoal> {
    const { data, error } = await supabase
      .from('store_performance_goals')
      .update(updates)
      .eq('id', goalId)
      .select()
      .single();

    if (error) {
      console.error('Error updating performance goal:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Update goal progress
  static async updateGoalProgress(
    goalId: string,
    currentValue: number
  ): Promise<boolean> {
    const { data, error } = await supabase
      .rpc('update_goal_progress', {
        goal_id: goalId,
        new_current_value: currentValue
      });

    if (error) {
      console.error('Error updating goal progress:', error);
      throw new Error(error.message);
    }

    return data;
  }

  // Delete performance goal
  static async deletePerformanceGoal(goalId: string): Promise<void> {
    const { error } = await supabase
      .from('store_performance_goals')
      .delete()
      .eq('id', goalId);

    if (error) {
      console.error('Error deleting performance goal:', error);
      throw new Error(error.message);
    }
  }

  // Calculate revenue for period
  static async calculateStoreRevenue(storeId: string): Promise<number> {
    const { data, error } = await supabase
      .rpc('calculate_store_revenue', {
        store_id: storeId
      });

    if (error) {
      console.error('Error calculating store revenue:', error);
      throw new Error(error.message);
    }

    return data || 0;
  }

  // Get revenue trends
  static async getRevenueTrends(
    storeId: string,
    period: 'day' | 'week' | 'month' = 'day',
    days = 30
  ) {
    const endDate = new Date();
    const startDate = new Date();
    startDate.setDate(endDate.getDate() - days);

    const { data, error } = await supabase
      .from('store_analytics')
      .select('date, revenue, order_count, average_order_value')
      .eq('store_id', storeId)
      .gte('date', startDate.toISOString().split('T')[0])
      .lte('date', endDate.toISOString().split('T')[0])
      .order('date', { ascending: true });

    if (error) {
      console.error('Error fetching revenue trends:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get top performing products
  static async getTopProducts(
    storeId: string,
    limit = 10,
    period?: { startDate: string; endDate: string }
  ) {
    let query = supabase
      .from('order_items')
      .select(`
        product_id,
        sum(quantity) as total_quantity,
        sum(total_price) as total_revenue,
        products!inner (
          title,
          name,
          price,
          store_id
        )
      `)
      .eq('products.store_id', storeId);

    if (period) {
      query = query
        .gte('created_at', period.startDate)
        .lte('created_at', period.endDate);
    }

    const { data, error } = await query
      .group('product_id, products.title, products.name, products.price, products.store_id')
      .order('total_revenue', { ascending: false })
      .limit(limit);

    if (error) {
      console.error('Error fetching top products:', error);
      throw new Error(error.message);
    }

    return data || [];
  }

  // Get customer insights
  static async getCustomerInsights(storeId: string) {
    const { data: orders, error } = await supabase
      .from('orders')
      .select('user_id, total_amount, created_at')
      .eq('store_id', storeId)
      .not('status', 'in', '(cancelled,refunded)');

    if (error) {
      console.error('Error fetching customer insights:', error);
      throw new Error(error.message);
    }

    // Process customer data
    const customerMap = new Map();
    orders?.forEach(order => {
      const existing = customerMap.get(order.user_id) || {
        totalSpent: 0,
        orderCount: 0,
        firstOrder: order.created_at,
        lastOrder: order.created_at
      };

      customerMap.set(order.user_id, {
        totalSpent: existing.totalSpent + order.total_amount,
        orderCount: existing.orderCount + 1,
        firstOrder: order.created_at < existing.firstOrder ? order.created_at : existing.firstOrder,
        lastOrder: order.created_at > existing.lastOrder ? order.created_at : existing.lastOrder
      });
    });

    const customers = Array.from(customerMap.values());
    
    return {
      totalCustomers: customers.length,
      newCustomers: customers.filter(c => 
        new Date(c.firstOrder) > new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
      ).length,
      returningCustomers: customers.filter(c => c.orderCount > 1).length,
      averageOrderValue: customers.length ? 
        customers.reduce((sum, c) => sum + c.totalSpent, 0) / 
        customers.reduce((sum, c) => sum + c.orderCount, 0) : 0,
      averageCustomerValue: customers.length ?
        customers.reduce((sum, c) => sum + c.totalSpent, 0) / customers.length : 0
    };
  }

  // Generate daily analytics
  static async generateDailyAnalytics(storeId: string, date: string): Promise<void> {
    const { error } = await supabase
      .rpc('calculate_daily_analytics', {
        target_store_id: storeId,
        target_date: date
      });

    if (error) {
      console.error('Error generating daily analytics:', error);
      throw new Error(error.message);
    }
  }
} 