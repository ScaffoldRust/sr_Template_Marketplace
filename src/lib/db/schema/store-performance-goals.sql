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

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_store_goals_store_id ON store_performance_goals(store_id);
CREATE INDEX IF NOT EXISTS idx_store_goals_type ON store_performance_goals(goal_type);
CREATE INDEX IF NOT EXISTS idx_store_goals_period ON store_performance_goals(time_period);
CREATE INDEX IF NOT EXISTS idx_store_goals_dates ON store_performance_goals(start_date, end_date); 