# Seller Dashboard Data Layer

This documentation covers the complete implementation of the Seller Dashboard Data Layer for StellarMarket, including database schema, APIs, views, functions, and security policies.

## Overview

The Seller Dashboard Data Layer provides comprehensive analytics and order management capabilities for sellers, including:

- **Store Management**: Complete CRUD operations for store entities
- **Order Management**: Order tracking, status updates, and filtering
- **Analytics**: Revenue tracking, performance metrics, and goal setting
- **Security**: Row-level security policies and role-based access control
- **Performance**: Optimized queries, indexes, and automated metrics updates

## Database Schema

### Core Tables

#### 1. Stores
Primary entity for marketplace sellers.

**Key Features:**
- Complete store profile information
- Automated metric calculations (revenue, ratings, product counts)
- Performance goal tracking
- Social media integration

```sql
CREATE TABLE stores (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID REFERENCES auth.users(id),
    slug TEXT UNIQUE NOT NULL,
    -- ... additional fields
    revenue_total DECIMAL(12, 2) DEFAULT 0,
    active_product_count INTEGER DEFAULT 0,
    pending_order_count INTEGER DEFAULT 0,
    average_rating DECIMAL(3, 2) DEFAULT 0,
    -- ... performance metrics
);
```

#### 2. Orders
Complete order management system.

**Key Features:**
- Multi-status order tracking
- Financial calculations (subtotal, tax, shipping, discounts)
- Address management (shipping/billing)
- Payment integration ready

```sql
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES auth.users(id),
    store_id UUID REFERENCES stores(id),
    status TEXT DEFAULT 'created' CHECK (
        status IN ('created', 'processing', 'paid', 'shipped', 'delivered', 'cancelled', 'refunded')
    ),
    total_amount DECIMAL(12, 2) NOT NULL,
    -- ... additional fields
);
```

#### 3. Store Analytics
Daily analytics aggregation for performance tracking.

**Key Features:**
- Daily revenue and order tracking
- Customer acquisition metrics
- Conversion rate tracking
- Store visit analytics

```sql
CREATE TABLE store_analytics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id UUID REFERENCES stores(id),
    date DATE NOT NULL,
    revenue DECIMAL(12, 2) DEFAULT 0,
    order_count INTEGER DEFAULT 0,
    new_customers INTEGER DEFAULT 0,
    returning_customers INTEGER DEFAULT 0,
    average_order_value DECIMAL(12, 2) DEFAULT 0,
    conversion_rate DECIMAL(5, 4) DEFAULT 0,
    view_count INTEGER DEFAULT 0,
    UNIQUE(store_id, date)
);
```

#### 4. Store Performance Goals
Goal setting and tracking system.

**Key Features:**
- Multiple goal types (sales, customers, reviews, conversion, AOV)
- Flexible time periods (daily, weekly, monthly, quarterly, yearly)
- Progress tracking with automated updates

```sql
CREATE TABLE store_performance_goals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id UUID REFERENCES stores(id),
    goal_type TEXT CHECK (goal_type IN ('sales', 'customers', 'reviews', 'conversion', 'aov')),
    target_value DECIMAL(12, 2) NOT NULL,
    current_value DECIMAL(12, 2) DEFAULT 0,
    time_period TEXT CHECK (time_period IN ('daily', 'weekly', 'monthly', 'quarterly', 'yearly')),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    UNIQUE(store_id, goal_type, time_period, start_date)
);
```

#### 5. Order Status History
Complete audit trail for order status changes.

**Key Features:**
- Automatic status change tracking
- User attribution for changes
- Timestamped audit trail
- Additional notes support

```sql
CREATE TABLE order_status_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID REFERENCES orders(id),
    status TEXT NOT NULL,
    changed_at TIMESTAMPTZ DEFAULT NOW(),
    changed_by UUID REFERENCES auth.users(id),
    notes TEXT
);
```

### Supporting Tables

- **Order Items**: Individual line items for each order
- **Reviews**: Customer reviews for products and stores

## Database Views

### 1. Seller Dashboard View
Comprehensive overview data for dashboard display.

```sql
CREATE VIEW seller_dashboard_view AS
SELECT 
    s.id as store_id,
    s.name as store_name,
    s.owner_id,
    COALESCE(SUM(o.total_amount) FILTER (WHERE o.status NOT IN ('cancelled', 'refunded')), 0) as total_revenue,
    COUNT(DISTINCT p.id) FILTER (WHERE p.active = true) as active_products,
    COUNT(DISTINCT o.id) FILTER (WHERE o.status = 'processing') as pending_orders,
    -- ... additional aggregated metrics
FROM stores s
LEFT JOIN products p ON s.id = p.store_id
LEFT JOIN orders o ON s.id = o.store_id
LEFT JOIN reviews r ON s.id = r.store_id
GROUP BY s.id, s.name, s.owner_id, s.monthly_sales_goal, s.customer_goal, s.review_goal, s.last_updated;
```

### 2. Seller Recent Orders View
Detailed order information with customer and product details.

### 3. Store Performance View
Goal tracking with calculated progress percentages.

### 4. Store Analytics Summary View
Analytics data with week-over-week and month-over-month comparisons.

## Database Functions

### 1. Metric Calculation Functions
- `calculate_store_revenue(store_id)`: Real-time revenue calculation
- `update_store_metrics()`: Automated metrics update (trigger function)
- `calculate_performance_percentage()`: Goal progress calculation

### 2. Analytics Functions
- `get_store_analytics()`: Retrieve analytics for date range
- `calculate_daily_analytics()`: Daily analytics aggregation
- `update_goal_progress()`: Manual goal progress updates

### 3. Audit Functions
- `track_order_status_change()`: Automatic status change logging

## Database Triggers

### Automated Metric Updates
- **Order Changes**: Updates store metrics when orders are created, updated, or deleted
- **Product Changes**: Updates product counts when products are added/removed/activated
- **Review Changes**: Updates rating metrics when reviews are added/updated/deleted

### Audit Trail
- **Order Status Changes**: Automatically logs all status changes with timestamps and user attribution

### Analytics Automation
- **Daily Analytics**: Automatically calculates and stores daily analytics when orders are processed

## Security Implementation

### Row-Level Security (RLS) Policies

#### Store Access
- Sellers can only access their own stores
- Admins can access all stores
- Authenticated users can create stores (with ownership validation)

#### Order Access
- Sellers can view/manage orders for their stores
- Customers can view/manage their own orders
- Admins have full access

#### Analytics Access
- Sellers can only view analytics for their stores
- System processes can manage all analytics
- Admins have read access to all analytics

#### Goal Management
- Sellers can fully manage goals for their stores only

### Security Functions
- `is_admin()`: Centralized admin role checking
- Role-based access control integration

## API Layer

### 1. Stores API (`src/lib/api/stores.ts`)

**Features:**
- Complete CRUD operations
- Store profile management
- Metric retrieval and updates
- Error handling and validation

**Key Methods:**
```typescript
class StoresAPI {
    async getStore(storeId: string): Promise<Store>
    async updateStore(storeId: string, updates: Partial<Store>): Promise<Store>
    async getStoreMetrics(storeId: string): Promise<StoreMetrics>
    async updateStoreMetrics(storeId: string): Promise<void>
}
```

### 2. Orders API (`src/lib/api/orders.ts`)

**Features:**
- Advanced filtering and pagination
- Status management
- Order creation and updates
- Performance optimized queries

**Key Methods:**
```typescript
class OrdersAPI {
    async getOrders(storeId: string, filters?: OrderFilters): Promise<PaginatedOrders>
    async getOrder(orderId: string): Promise<Order>
    async updateOrderStatus(orderId: string, status: OrderStatus): Promise<Order>
    async getOrderHistory(orderId: string): Promise<OrderStatusHistory[]>
}
```

### 3. Seller Dashboard API (`src/lib/api/seller-dashboard.ts`)

**Features:**
- Dashboard overview data
- Analytics retrieval
- Performance goal management
- Customer insights

**Key Methods:**
```typescript
class SellerDashboardAPI {
    async getDashboardOverview(storeId: string): Promise<DashboardOverview>
    async getStoreAnalytics(storeId: string, params: AnalyticsParams): Promise<StoreAnalytics[]>
    async getPerformanceGoals(storeId: string): Promise<PerformanceGoal[]>
    async updatePerformanceGoal(goalId: string, updates: Partial<PerformanceGoal>): Promise<PerformanceGoal>
}
```

## Performance Optimizations

### Database Indexes
Comprehensive indexing strategy for optimal query performance:

```sql
-- Core entity indexes
CREATE INDEX idx_stores_owner_id ON stores(owner_id);
CREATE INDEX idx_orders_store_id ON orders(store_id);
CREATE INDEX idx_orders_store_status ON orders(store_id, status);

-- Analytics indexes
CREATE INDEX idx_store_analytics_store_date ON store_analytics(store_id, date);
CREATE INDEX idx_store_goals_store_id ON store_performance_goals(store_id);

-- Performance indexes
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_products_store_id ON products(store_id);
```

### Query Optimizations
- Filtered aggregations using `FILTER` clauses
- Optimized JOIN strategies in views
- Efficient pagination with offset/limit
- Cached metric calculations in store table

### Automated Updates
- Trigger-based metric updates for real-time accuracy
- Batch analytics processing
- Optimized upsert operations

## Data Migration

### Migration File
Complete migration script: `src/lib/db/migrations/seller-dashboard-migration.sql`

**Includes:**
- Table creation with proper constraints
- Index creation for performance
- Function and trigger setup
- RLS policy implementation
- Safe column additions to existing tables

### Migration Safety
- `IF NOT EXISTS` clauses for safe re-runs
- Proper foreign key relationships
- Data validation constraints
- Rollback considerations

## Error Handling

### Database Level
- Constraint violations with meaningful messages
- Foreign key integrity enforcement
- Data type validation
- Range checks for numerical values

### API Level
- Comprehensive error typing
- Detailed error messages
- Proper HTTP status codes
- Error logging and monitoring

### Client Level
- Type-safe error handling
- User-friendly error messages
- Fallback data states
- Loading state management

## Usage Examples

### Getting Dashboard Data
```typescript
import { SellerDashboardAPI } from '@/lib/api/seller-dashboard';

const api = new SellerDashboardAPI();
const overview = await api.getDashboardOverview(storeId);
```

### Managing Orders
```typescript
import { OrdersAPI } from '@/lib/api/orders';

const api = new OrdersAPI();
const orders = await api.getOrders(storeId, {
    status: 'processing',
    limit: 20,
    offset: 0
});
```

### Tracking Analytics
```typescript
import { SellerDashboardAPI } from '@/lib/api/seller-dashboard';

const api = new SellerDashboardAPI();
const analytics = await api.getStoreAnalytics(storeId, {
    startDate: '2024-01-01',
    endDate: '2024-01-31',
    groupBy: 'day'
});
```

## Testing Considerations

### Database Tests
- Schema validation
- Constraint testing
- Trigger functionality
- RLS policy verification
- Performance benchmarking

### API Tests
- CRUD operation testing
- Error handling verification
- Authentication testing
- Performance testing
- Integration testing

### Security Tests
- RLS policy enforcement
- Access control verification
- Data isolation testing
- SQL injection prevention

## Monitoring and Maintenance

### Performance Monitoring
- Query performance tracking
- Index usage analysis
- Trigger execution monitoring
- API response time tracking

### Data Quality
- Metric accuracy validation
- Data consistency checks
- Referential integrity monitoring
- Analytics data verification

### Security Monitoring
- Access pattern analysis
- Failed authentication tracking
- Privilege escalation detection
- Data access auditing

## Future Enhancements

### Potential Improvements
1. **Real-time Analytics**: WebSocket integration for live updates
2. **Advanced Reporting**: Custom report generation
3. **Predictive Analytics**: Machine learning integration
4. **Multi-store Management**: Support for sellers with multiple stores
5. **Advanced Goals**: Complex goal types and dependencies
6. **Customer Segmentation**: Advanced customer analytics
7. **Inventory Integration**: Stock level tracking and alerts
8. **Performance Benchmarking**: Industry comparison metrics

### Scalability Considerations
- Database sharding strategies
- Read replica implementation
- Caching layer integration
- Background job processing
- Archive strategy for historical data

## Conclusion

This implementation provides a robust, secure, and performant foundation for seller dashboard functionality in StellarMarket. The architecture supports both current requirements and future growth, with comprehensive error handling, security measures, and performance optimizations. 