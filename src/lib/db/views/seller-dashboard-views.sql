-- Store Dashboard View
CREATE OR REPLACE VIEW seller_dashboard_view AS
SELECT 
    s.id as store_id,
    s.name as store_name,
    s.owner_id,
    COALESCE(SUM(o.total_amount) FILTER (WHERE o.status NOT IN ('cancelled', 'refunded')), 0) as total_revenue,
    COUNT(DISTINCT p.id) FILTER (WHERE p.active = true) as active_products,
    COUNT(DISTINCT o.id) FILTER (WHERE o.status = 'processing') as pending_orders,
    COALESCE(AVG(r.rating), 0) as average_rating,
    COUNT(DISTINCT r.id) as rating_count,
    COUNT(DISTINCT o.id) FILTER (WHERE o.status NOT IN ('cancelled', 'refunded')) as total_orders,
    s.monthly_sales_goal,
    s.customer_goal,
    s.review_goal,
    s.last_updated
FROM stores s
LEFT JOIN products p ON s.id = p.store_id
LEFT JOIN orders o ON s.id = o.store_id
LEFT JOIN reviews r ON s.id = r.store_id
GROUP BY s.id, s.name, s.owner_id, s.monthly_sales_goal, s.customer_goal, s.review_goal, s.last_updated;

-- Recent Orders View
CREATE OR REPLACE VIEW seller_recent_orders_view AS
SELECT 
    o.id as order_id,
    o.store_id,
    o.total_amount,
    o.status,
    o.created_at,
    o.updated_at,
    o.tracking_number,
    u.raw_user_meta_data->>'display_name' as customer_name,
    u.email as customer_email,
    oi.product_id,
    oi.quantity,
    oi.price_at_purchase,
    oi.total_price,
    COALESCE(p.name, p.title) as product_name,
    p.variant as product_variant,
    s.name as store_name
FROM orders o
JOIN order_items oi ON o.id = oi.order_id
JOIN products p ON oi.product_id = p.id
JOIN stores s ON o.store_id = s.id
JOIN auth.users u ON o.user_id = u.id
ORDER BY o.created_at DESC;

-- Store Performance View
CREATE OR REPLACE VIEW store_performance_view AS
SELECT
    s.id as store_id,
    s.name as store_name,
    g.id as goal_id,
    g.goal_type,
    g.target_value,
    g.current_value,
    CASE 
        WHEN g.target_value > 0 THEN 
            LEAST(ROUND((g.current_value / g.target_value) * 100), 100)
        ELSE 0
    END as percentage,
    g.time_period,
    g.start_date,
    g.end_date,
    g.created_at,
    g.updated_at
FROM stores s
JOIN store_performance_goals g ON s.id = g.store_id;

-- Store Analytics Summary View
CREATE OR REPLACE VIEW store_analytics_summary_view AS
SELECT
    sa.store_id,
    s.name as store_name,
    sa.date,
    sa.revenue,
    sa.order_count,
    sa.new_customers,
    sa.returning_customers,
    sa.average_order_value,
    sa.conversion_rate,
    sa.view_count,
    -- Calculate week-over-week and month-over-month changes
    LAG(sa.revenue, 7) OVER (PARTITION BY sa.store_id ORDER BY sa.date) as revenue_week_ago,
    LAG(sa.revenue, 30) OVER (PARTITION BY sa.store_id ORDER BY sa.date) as revenue_month_ago,
    LAG(sa.order_count, 7) OVER (PARTITION BY sa.store_id ORDER BY sa.date) as orders_week_ago,
    LAG(sa.order_count, 30) OVER (PARTITION BY sa.store_id ORDER BY sa.date) as orders_month_ago
FROM store_analytics sa
JOIN stores s ON sa.store_id = s.id
ORDER BY sa.store_id, sa.date DESC; 