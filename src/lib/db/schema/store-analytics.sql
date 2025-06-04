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

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_store_analytics_store_id ON store_analytics(store_id);
CREATE INDEX IF NOT EXISTS idx_store_analytics_date ON store_analytics(date);
CREATE INDEX IF NOT EXISTS idx_store_analytics_store_date ON store_analytics(store_id, date); 