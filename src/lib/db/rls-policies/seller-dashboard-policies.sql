-- Enable RLS on all new tables
ALTER TABLE stores ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE store_analytics ENABLE ROW LEVEL SECURITY;
ALTER TABLE store_performance_goals ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_status_history ENABLE ROW LEVEL SECURITY;

-- Stores Table Policies
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

-- Orders Table Policies
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
    )
    WITH CHECK (
        -- Only allow updating specific fields
        (OLD.store_id = NEW.store_id) AND
        (OLD.user_id = NEW.user_id) AND
        (OLD.total_amount = NEW.total_amount)
    );

CREATE POLICY "Customers can update their own orders"
    ON orders FOR UPDATE
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

-- Order Items Table Policies
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

-- Store Analytics Table Policies
CREATE POLICY "Sellers can view their own analytics"
    ON store_analytics FOR SELECT
    USING (
        auth.uid() IN (
            SELECT owner_id FROM stores WHERE id = store_id
        )
    );

CREATE POLICY "System can manage analytics"
    ON store_analytics FOR ALL
    USING (true)
    WITH CHECK (true);

-- Store Performance Goals Table Policies
CREATE POLICY "Sellers can manage their own goals"
    ON store_performance_goals FOR ALL
    USING (
        auth.uid() IN (
            SELECT owner_id FROM stores WHERE id = store_id
        )
    )
    WITH CHECK (
        auth.uid() IN (
            SELECT owner_id FROM stores WHERE id = store_id
        )
    );

-- Order Status History Table Policies
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

-- Additional policies for admin access
CREATE POLICY "Admins can view all stores"
    ON stores FOR SELECT
    USING (is_admin());

CREATE POLICY "Admins can manage all orders"
    ON orders FOR ALL
    USING (is_admin())
    WITH CHECK (is_admin());

CREATE POLICY "Admins can view all analytics"
    ON store_analytics FOR SELECT
    USING (is_admin()); 