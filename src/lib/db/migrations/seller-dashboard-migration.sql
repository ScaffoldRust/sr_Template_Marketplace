-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create admin check function if it doesn't exist
CREATE OR REPLACE FUNCTION is_admin()
RETURNS BOOLEAN AS $$
BEGIN
    RETURN auth.uid() IN (
        SELECT user_id FROM user_roles WHERE role = 'admin'
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create reviews table if it doesn't exist (needed for store metrics)
CREATE TABLE IF NOT EXISTS reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID REFERENCES products(id) ON DELETE CASCADE,
    store_id UUID,
    user_id UUID REFERENCES auth.users(id),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(product_id, user_id)
);

-- Stores Table
CREATE TABLE IF NOT EXISTS stores (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    slug TEXT UNIQUE NOT NULL,
    logo_url TEXT,
    banner_url TEXT,
    contact_email TEXT,
    phone TEXT,
    address TEXT,
    city TEXT,
    state TEXT,
    country TEXT,
    postal_code TEXT,
    website_url TEXT,
    social_media JSONB DEFAULT '{}',
    revenue_total DECIMAL(12, 2) DEFAULT 0,
    active_product_count INTEGER DEFAULT 0,
    pending_order_count INTEGER DEFAULT 0,
    average_rating DECIMAL(3, 2) DEFAULT 0,
    rating_count INTEGER DEFAULT 0,
    monthly_sales_goal DECIMAL(12, 2),
    customer_goal INTEGER,
    review_goal INTEGER,
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Orders Table
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    store_id UUID REFERENCES stores(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'created' CHECK (
        status IN ('created', 'processing', 'paid', 'shipped', 'delivered', 'cancelled', 'refunded')
    ),
    total_amount DECIMAL(12, 2) NOT NULL CHECK (total_amount >= 0),
    subtotal DECIMAL(12, 2) NOT NULL CHECK (subtotal >= 0),
    tax_amount DECIMAL(12, 2) DEFAULT 0 CHECK (tax_amount >= 0),
    shipping_amount DECIMAL(12, 2) DEFAULT 0 CHECK (shipping_amount >= 0),
    discount_amount DECIMAL(12, 2) DEFAULT 0 CHECK (discount_amount >= 0),
    currency TEXT DEFAULT 'XLM',
    payment_method TEXT,
    payment_id TEXT,
    shipping_address JSONB,
    billing_address JSONB,
    tracking_number TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Order Items Table
CREATE TABLE IF NOT EXISTS order_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    price_at_purchase DECIMAL(12, 2) NOT NULL CHECK (price_at_purchase >= 0),
    total_price DECIMAL(12, 2) NOT NULL CHECK (total_price >= 0),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Store Analytics Table
CREATE TABLE IF NOT EXISTS store_analytics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id UUID REFERENCES stores(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    revenue DECIMAL(12, 2) DEFAULT 0 CHECK (revenue >= 0),
    order_count INTEGER DEFAULT 0 CHECK (order_count >= 0),
    new_customers INTEGER DEFAULT 0 CHECK (new_customers >= 0),
    returning_customers INTEGER DEFAULT 0 CHECK (returning_customers >= 0),
    average_order_value DECIMAL(12, 2) DEFAULT 0 CHECK (average_order_value >= 0),
    conversion_rate DECIMAL(5, 4) DEFAULT 0 CHECK (conversion_rate >= 0 AND conversion_rate <= 1),
    view_count INTEGER DEFAULT 0 CHECK (view_count >= 0),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(store_id, date)
);

-- Store Performance Goals Table
CREATE TABLE IF NOT EXISTS store_performance_goals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id UUID REFERENCES stores(id) ON DELETE CASCADE,
    goal_type TEXT NOT NULL CHECK (
        goal_type IN ('sales', 'customers', 'reviews', 'conversion', 'aov')
    ),
    target_value DECIMAL(12, 2) NOT NULL CHECK (target_value > 0),
    current_value DECIMAL(12, 2) DEFAULT 0 CHECK (current_value >= 0),
    time_period TEXT NOT NULL CHECK (
        time_period IN ('daily', 'weekly', 'monthly', 'quarterly', 'yearly')
    ),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CHECK (end_date > start_date),
    UNIQUE(store_id, goal_type, time_period, start_date)
);

-- Order Status History Table
CREATE TABLE IF NOT EXISTS order_status_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (
        status IN ('created', 'processing', 'paid', 'shipped', 'delivered', 'cancelled', 'refunded')
    ),
    changed_at TIMESTAMPTZ DEFAULT NOW(),
    changed_by UUID REFERENCES auth.users(id),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Update products table to include store_id if not present
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'products' AND column_name = 'store_id'
    ) THEN
        ALTER TABLE products ADD COLUMN store_id UUID REFERENCES stores(id);
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'products' AND column_name = 'active'
    ) THEN
        ALTER TABLE products ADD COLUMN active BOOLEAN DEFAULT true;
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'products' AND column_name = 'variant'
    ) THEN
        ALTER TABLE products ADD COLUMN variant TEXT;
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'products' AND column_name = 'name'
    ) THEN
        ALTER TABLE products ADD COLUMN name TEXT;
    END IF;
    
    -- Add store_id to reviews if not present
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'reviews' AND column_name = 'store_id'
    ) THEN
        ALTER TABLE reviews ADD COLUMN store_id UUID REFERENCES stores(id);
    END IF;
END
$$;

-- Create performance indexes
CREATE INDEX IF NOT EXISTS idx_stores_owner_id ON stores(owner_id);
CREATE INDEX IF NOT EXISTS idx_stores_slug ON stores(slug);
CREATE INDEX IF NOT EXISTS idx_stores_updated ON stores(last_updated);
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_store_id ON orders(store_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_store_status ON orders(store_id, status);
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON order_items(product_id);
CREATE INDEX IF NOT EXISTS idx_store_analytics_store_id ON store_analytics(store_id);
CREATE INDEX IF NOT EXISTS idx_store_analytics_date ON store_analytics(date);
CREATE INDEX IF NOT EXISTS idx_store_analytics_store_date ON store_analytics(store_id, date);
CREATE INDEX IF NOT EXISTS idx_store_goals_store_id ON store_performance_goals(store_id);
CREATE INDEX IF NOT EXISTS idx_store_goals_type ON store_performance_goals(goal_type);
CREATE INDEX IF NOT EXISTS idx_store_goals_period ON store_performance_goals(time_period);
CREATE INDEX IF NOT EXISTS idx_store_goals_dates ON store_performance_goals(start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_order_status_history_order_id ON order_status_history(order_id);
CREATE INDEX IF NOT EXISTS idx_order_status_history_changed_at ON order_status_history(changed_at);
CREATE INDEX IF NOT EXISTS idx_order_status_history_status ON order_status_history(status);
CREATE INDEX IF NOT EXISTS idx_reviews_store_id ON reviews(store_id);
CREATE INDEX IF NOT EXISTS idx_products_store_id ON products(store_id);

-- Create database functions
CREATE OR REPLACE FUNCTION calculate_store_revenue(store_id UUID)
RETURNS DECIMAL AS $$
DECLARE
    total DECIMAL;
BEGIN
    SELECT COALESCE(SUM(total_amount), 0)
    INTO total
    FROM orders
    WHERE orders.store_id = calculate_store_revenue.store_id
        AND status NOT IN ('cancelled', 'refunded');
    
    RETURN total;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Update Store Metrics Function
CREATE OR REPLACE FUNCTION update_store_metrics()
RETURNS TRIGGER AS $$
BEGIN
    -- Handle both INSERT and UPDATE operations
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        UPDATE stores
        SET 
            revenue_total = calculate_store_revenue(NEW.store_id),
            active_product_count = (
                SELECT COUNT(*) FROM products 
                WHERE store_id = NEW.store_id AND active = true
            ),
            pending_order_count = (
                SELECT COUNT(*) FROM orders 
                WHERE store_id = NEW.store_id AND status = 'processing'
            ),
            average_rating = (
                SELECT COALESCE(AVG(rating), 0) FROM reviews 
                WHERE store_id = NEW.store_id
            ),
            rating_count = (
                SELECT COUNT(*) FROM reviews 
                WHERE store_id = NEW.store_id
            ),
            last_updated = NOW()
        WHERE id = NEW.store_id;
        
        RETURN NEW;
    END IF;
    
    -- Handle DELETE operations
    IF TG_OP = 'DELETE' THEN
        UPDATE stores
        SET 
            revenue_total = calculate_store_revenue(OLD.store_id),
            active_product_count = (
                SELECT COUNT(*) FROM products 
                WHERE store_id = OLD.store_id AND active = true
            ),
            pending_order_count = (
                SELECT COUNT(*) FROM orders 
                WHERE store_id = OLD.store_id AND status = 'processing'
            ),
            average_rating = (
                SELECT COALESCE(AVG(rating), 0) FROM reviews 
                WHERE store_id = OLD.store_id
            ),
            rating_count = (
                SELECT COUNT(*) FROM reviews 
                WHERE store_id = OLD.store_id
            ),
            last_updated = NOW()
        WHERE id = OLD.store_id;
        
        RETURN OLD;
    END IF;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Other helper functions
CREATE OR REPLACE FUNCTION calculate_performance_percentage(
    current_value DECIMAL,
    target_value DECIMAL
) RETURNS INTEGER AS $$
BEGIN
    IF target_value IS NULL OR target_value = 0 THEN
        RETURN 0;
    END IF;
    
    RETURN LEAST(ROUND((current_value / target_value) * 100)::INTEGER, 100);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION update_goal_progress(
    goal_id UUID,
    new_current_value DECIMAL
) RETURNS BOOLEAN AS $$
BEGIN
    UPDATE store_performance_goals
    SET 
        current_value = new_current_value,
        updated_at = NOW()
    WHERE id = goal_id;
    
    RETURN FOUND;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION track_order_status_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Only track if status actually changed
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO order_status_history (
            order_id,
            status,
            changed_at,
            changed_by,
            notes
        ) VALUES (
            NEW.id,
            NEW.status,
            NOW(),
            auth.uid(),
            'Status changed from ' || COALESCE(OLD.status, 'null') || ' to ' || NEW.status
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION get_store_analytics(
    store_id UUID,
    start_date DATE,
    end_date DATE
) RETURNS TABLE(
    date DATE,
    revenue DECIMAL,
    order_count INTEGER,
    new_customers INTEGER,
    returning_customers INTEGER,
    average_order_value DECIMAL,
    conversion_rate DECIMAL,
    view_count INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        sa.date,
        sa.revenue,
        sa.order_count,
        sa.new_customers,
        sa.returning_customers,
        sa.average_order_value,
        sa.conversion_rate,
        sa.view_count
    FROM store_analytics sa
    WHERE sa.store_id = get_store_analytics.store_id
        AND sa.date BETWEEN start_date AND end_date
    ORDER BY sa.date;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION calculate_daily_analytics(
    target_store_id UUID,
    target_date DATE
) RETURNS VOID AS $$
DECLARE
    daily_revenue DECIMAL;
    daily_orders INTEGER;
    daily_new_customers INTEGER;
    daily_returning_customers INTEGER;
    daily_aov DECIMAL;
    daily_views INTEGER;
BEGIN
    -- Calculate daily revenue
    SELECT COALESCE(SUM(total_amount), 0)
    INTO daily_revenue
    FROM orders
    WHERE store_id = target_store_id
        AND DATE(created_at) = target_date
        AND status NOT IN ('cancelled', 'refunded');
    
    -- Calculate daily order count
    SELECT COUNT(*)
    INTO daily_orders
    FROM orders
    WHERE store_id = target_store_id
        AND DATE(created_at) = target_date
        AND status NOT IN ('cancelled', 'refunded');
    
    -- Calculate average order value
    daily_aov := CASE 
        WHEN daily_orders > 0 THEN daily_revenue / daily_orders 
        ELSE 0 
    END;
    
    -- For now, set placeholders for customer metrics and views
    -- These would need to be calculated based on actual user tracking
    daily_new_customers := 0;
    daily_returning_customers := 0;
    daily_views := 0;
    
    -- Insert or update analytics record
    INSERT INTO store_analytics (
        store_id,
        date,
        revenue,
        order_count,
        new_customers,
        returning_customers,
        average_order_value,
        conversion_rate,
        view_count
    ) VALUES (
        target_store_id,
        target_date,
        daily_revenue,
        daily_orders,
        daily_new_customers,
        daily_returning_customers,
        daily_aov,
        0, -- conversion_rate placeholder
        daily_views
    )
    ON CONFLICT (store_id, date)
    DO UPDATE SET
        revenue = EXCLUDED.revenue,
        order_count = EXCLUDED.order_count,
        new_customers = EXCLUDED.new_customers,
        returning_customers = EXCLUDED.returning_customers,
        average_order_value = EXCLUDED.average_order_value,
        conversion_rate = EXCLUDED.conversion_rate,
        view_count = EXCLUDED.view_count,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create database views
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

-- Create triggers
DROP TRIGGER IF EXISTS trigger_update_store_metrics_on_order ON orders;
CREATE TRIGGER trigger_update_store_metrics_on_order
    AFTER INSERT OR UPDATE OR DELETE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION update_store_metrics();

DROP TRIGGER IF EXISTS trigger_update_store_metrics_on_product ON products;
CREATE TRIGGER trigger_update_store_metrics_on_product
    AFTER INSERT OR UPDATE OR DELETE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_store_metrics();

DROP TRIGGER IF EXISTS trigger_update_store_metrics_on_review ON reviews;
CREATE TRIGGER trigger_update_store_metrics_on_review
    AFTER INSERT OR UPDATE OR DELETE ON reviews
    FOR EACH ROW
    EXECUTE FUNCTION update_store_metrics();

DROP TRIGGER IF EXISTS trigger_track_order_status_change ON orders;
CREATE TRIGGER trigger_track_order_status_change
    AFTER UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION track_order_status_change();

DROP TRIGGER IF EXISTS trigger_calculate_daily_analytics ON orders;
CREATE TRIGGER trigger_calculate_daily_analytics
    AFTER INSERT OR UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION calculate_daily_analytics(NEW.store_id, DATE(NEW.created_at));

-- Enable RLS on all new tables
ALTER TABLE stores ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE store_analytics ENABLE ROW LEVEL SECURITY;
ALTER TABLE store_performance_goals ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_status_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE reviews ENABLE ROW LEVEL SECURITY;

-- Create RLS policies
CREATE POLICY "Sellers can view their own store"
    ON stores FOR SELECT
    USING (auth.uid() = owner_id);

CREATE POLICY "Sellers can update their own store"
    ON stores FOR UPDATE
    USING (auth.uid() = owner_id);

CREATE POLICY "Authenticated users can create stores"
    ON stores FOR INSERT
    WITH CHECK (auth.uid() = owner_id);

CREATE POLICY "Sellers can delete their own store"
    ON stores FOR DELETE
    USING (auth.uid() = owner_id);

CREATE POLICY "Sellers can view their store orders"
    ON orders FOR SELECT
    USING (
        store_id IN (
            SELECT id FROM stores WHERE owner_id = auth.uid()
        )
    );

CREATE POLICY "Customers can view their own orders"
    ON orders FOR SELECT
    USING (auth.uid() = user_id);

CREATE POLICY "Customers can create orders"
    ON orders FOR INSERT
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Sellers can update their store orders"
    ON orders FOR UPDATE
    USING (
        store_id IN (
            SELECT id FROM stores WHERE owner_id = auth.uid()
        )
    );

CREATE POLICY "Customers can update their own orders"
    ON orders FOR UPDATE
    USING (auth.uid() = user_id);

CREATE POLICY "Sellers can view their store order items"
    ON order_items FOR SELECT
    USING (
        order_id IN (
            SELECT o.id FROM orders o
            JOIN stores s ON o.store_id = s.id
            WHERE s.owner_id = auth.uid()
        )
    );

CREATE POLICY "Customers can view their own order items"
    ON order_items FOR SELECT
    USING (
        order_id IN (
            SELECT id FROM orders WHERE user_id = auth.uid()
        )
    );

CREATE POLICY "System can insert order items"
    ON order_items FOR INSERT
    WITH CHECK (true);

CREATE POLICY "Sellers can view their own analytics"
    ON store_analytics FOR SELECT
    USING (
        auth.uid() IN (
            SELECT owner_id FROM stores WHERE id = store_id
        )
    );

CREATE POLICY "System can manage analytics"
    ON store_analytics FOR ALL
    USING (true);

CREATE POLICY "Sellers can manage their own goals"
    ON store_performance_goals FOR ALL
    USING (
        auth.uid() IN (
            SELECT owner_id FROM stores WHERE id = store_id
        )
    );

CREATE POLICY "Sellers can view their store order history"
    ON order_status_history FOR SELECT
    USING (
        order_id IN (
            SELECT o.id FROM orders o
            JOIN stores s ON o.store_id = s.id
            WHERE s.owner_id = auth.uid()
        )
    );

CREATE POLICY "Customers can view their order history"
    ON order_status_history FOR SELECT
    USING (
        order_id IN (
            SELECT id FROM orders WHERE user_id = auth.uid()
        )
    );

CREATE POLICY "System can insert order history"
    ON order_status_history FOR INSERT
    WITH CHECK (true);

CREATE POLICY "Anyone can view reviews" 
    ON reviews FOR SELECT USING (true);

CREATE POLICY "Users can add reviews" 
    ON reviews FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own reviews" 
    ON reviews FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own reviews" 
    ON reviews FOR DELETE USING (auth.uid() = user_id);

-- Admin policies
CREATE POLICY "Admins can view all stores"
    ON stores FOR SELECT
    USING (is_admin());

CREATE POLICY "Admins can manage all orders"
    ON orders FOR ALL
    USING (is_admin());

CREATE POLICY "Admins can view all analytics"
    ON store_analytics FOR SELECT
    USING (is_admin()); 