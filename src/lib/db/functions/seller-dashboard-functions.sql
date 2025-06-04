-- Calculate Store Revenue Function
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

-- Calculate Performance Percentage Function
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

-- Update Goal Progress Function
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

-- Track Order Status Change Function
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

-- Get Store Analytics for Date Range Function
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

-- Calculate Daily Analytics Function
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