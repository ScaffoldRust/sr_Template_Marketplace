-- Create the calculate cart total function
CREATE OR REPLACE FUNCTION calculate_cart_total(user_id UUID)
RETURNS TABLE(total DECIMAL)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT COALESCE(SUM(p.price * c.quantity), 0)::DECIMAL(12,2) as total
    FROM cart_items c
    JOIN products p ON c.product_id = p.id
    WHERE c.user_id = calculate_cart_total.user_id;
END;
$$;

-- Function to search products based on various criteria
CREATE OR REPLACE FUNCTION search_products(
    search_query TEXT,
    category_id UUID DEFAULT NULL,
    min_price DECIMAL DEFAULT NULL,
    max_price DECIMAL DEFAULT NULL,
    min_rating DECIMAL DEFAULT NULL,
    limit_val INTEGER DEFAULT 10,
    offset_val INTEGER DEFAULT 0
    )
    
    RETURNS SETOF products
    LANGUAGE plpgsql SECURITY DEFINER
    AS $$ BEGIN RETURN QUERY
    SELECT p.* FROM products p
    WHERE 
        -- Full-text search on title and description
        (
        search_query IS NULL OR
        to_tsvector('english', p.title || ' ' || p.description) @@ to_tsquery('english', regexp_replace(search_query, '\s+', ':* & ', 'g') || ':*')
        )
        AND (category_id IS NULL OR p.category = category_id)

        AND (min_price IS NULL OR p.price >= min_price)

        AND (max_price IS NULL OR p.price <= max_price)

        AND (min_rating IS NULL OR p.rating >= min_rating)

    ORDER BY p.featured DESC,

        CASE WHEN search_query IS NOT NULL THEN
        ts_rank(to_tsvector('english', p.title || ' ' || p.description), 
                to_tsquery('english', regexp_replace(search_query, '\s+', ':* & ', 'g') || ':*'))

        ELSE 0 END DESC, p.rating DESC

    LIMIT limit_val

    OFFSET offset_val;
END;
$$;


-- Create a function to get related products
CREATE OR REPLACE FUNCTION get_related_products(
    product_id UUID,
    limit_val INTEGER DEFAULT 4
)
RETURNS SETOF products
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    product_category UUID;
BEGIN
    SELECT category INTO product_category
    FROM products
    WHERE id = get_related_products.product_id;
    
    RETURN QUERY
    SELECT p.*
    FROM products p
    WHERE p.category = product_category
    AND p.id != get_related_products.product_id
    ORDER BY p.rating DESC, p.created_at DESC
    LIMIT limit_val;
END;
$$;


-- Check if promotion_codes table exists, create it if not
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_name = 'promotion_codes'
    ) THEN
        CREATE TABLE promotion_codes (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            code TEXT UNIQUE NOT NULL,
            discount_type TEXT NOT NULL CHECK (discount_type IN ('percentage', 'fixed')),
            discount_value DECIMAL(12, 2) NOT NULL CHECK (discount_value > 0),
            min_order_amount DECIMAL(12, 2),
            max_discount DECIMAL(12, 2),
            valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            valid_to TIMESTAMPTZ NOT NULL,
            uses_limit INTEGER,
            uses INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );
        
        -- Enable RLS
        ALTER TABLE promotion_codes ENABLE ROW LEVEL SECURITY;
        
        -- RLS Policies - Only admins can manage, but any authenticated user can view valid promotions
        CREATE POLICY "Anyone can view valid promotions" 
            ON promotion_codes FOR SELECT USING (true);
            
        CREATE POLICY "Only admins can manage promotions" 
            ON promotion_codes FOR ALL USING (is_admin());
    END IF;
    
    -- Create user_promotions table if it doesn't exist
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_name = 'user_promotions'
    ) THEN
        CREATE TABLE user_promotions (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            user_id UUID REFERENCES auth.users(id),
            promotion_code TEXT REFERENCES promotion_codes(code),
            used_at TIMESTAMPTZ DEFAULT NOW(),
            discount_amount DECIMAL(12, 2) NOT NULL
        );
        
        -- Enable RLS
        ALTER TABLE user_promotions ENABLE ROW LEVEL SECURITY;
        
        -- RLS Policies
        CREATE POLICY "Users can view their own promotion usage" 
            ON user_promotions FOR SELECT USING (auth.uid() = user_id);
            
        CREATE POLICY "System can insert promotion usage" 
            ON user_promotions FOR INSERT WITH CHECK (true);
            
        CREATE POLICY "Admins can view all promotion usage" 
            ON user_promotions FOR SELECT USING (is_admin());
    END IF;
END
$$;

-- Create a function to handle promotion codes
CREATE OR REPLACE FUNCTION apply_promotion_code(
    user_id UUID,
    code TEXT
)
RETURNS TABLE(
    success BOOLEAN,
    message TEXT,
    discount_amount DECIMAL,
    cart_total DECIMAL
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    promo RECORD;
    cart_subtotal DECIMAL;
    discount DECIMAL := 0;
BEGIN
    -- Check if promotion code exists and is valid
    SELECT * INTO promo
    FROM promotion_codes
    WHERE promotion_codes.code = apply_promotion_code.code
    AND now() BETWEEN valid_from AND valid_to
    AND (uses_limit IS NULL OR uses < uses_limit);
    
    -- If promotion not found or invalid
    IF NOT FOUND THEN
        RETURN QUERY SELECT false, 'Invalid or expired promotion code', 0::DECIMAL, 0::DECIMAL;
        RETURN;
    END IF;
    
    -- Get cart subtotal
    SELECT COALESCE(SUM(p.price * c.quantity), 0)::DECIMAL(12,2) INTO cart_subtotal
    FROM cart_items c
    JOIN products p ON c.product_id = p.id
    WHERE c.user_id = apply_promotion_code.user_id;
    
    -- Check minimum order amount if applicable
    IF promo.min_order_amount IS NOT NULL AND cart_subtotal < promo.min_order_amount THEN
        RETURN QUERY 
            SELECT false, 
                'This promotion requires a minimum order of ' || promo.min_order_amount::TEXT, 
                0::DECIMAL, 
                cart_subtotal;
        RETURN;
    END IF;
    
    -- Calculate discount based on type
    IF promo.discount_type = 'percentage' THEN
        discount := (cart_subtotal * promo.discount_value / 100)::DECIMAL(12,2);
    ELSE
        -- Fixed amount discount
        discount := LEAST(promo.discount_value, cart_subtotal)::DECIMAL(12,2);
    END IF;
    
    -- Apply maximum discount if applicable
    IF promo.max_discount IS NOT NULL THEN
        discount := LEAST(discount, promo.max_discount);
    END IF;
    
    -- Record usage
    UPDATE promotion_codes
    SET uses = uses + 1
    WHERE code = apply_promotion_code.code;
    
    INSERT INTO user_promotions (user_id, promotion_code, used_at, discount_amount)
    VALUES (apply_promotion_code.user_id, apply_promotion_code.code, now(), discount);
    
    RETURN QUERY 
    SELECT true, 
        'Promotion applied successfully', 
        discount, 
        (cart_subtotal - discount)::DECIMAL(12,2);
END;
$$;
