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


-- Create database trigger to maintain category paths for hierarchical categories
CREATE OR REPLACE FUNCTION update_category_path()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    parent_path TEXT;
BEGIN
    -- If this is a top-level category (no parent)
    IF NEW.parent_id IS NULL THEN
    NEW.path = NEW.id::TEXT;
    ELSE
    -- Get parent's path
    SELECT path INTO parent_path
    FROM categories
    WHERE id = NEW.parent_id;
    
    -- Append this category's ID to the path
    NEW.path = parent_path || '.' || NEW.id::TEXT;
    END IF;
    
    RETURN NEW;
END;
$$;

-- Add path column to categories table if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
    SELECT 1 
    FROM information_schema.columns 
    WHERE table_name = 'categories' AND column_name = 'path'
    ) THEN
    ALTER TABLE categories ADD COLUMN path TEXT;
    END IF;
END$$;

-- Create trigger for category path
CREATE TRIGGER trigger_update_category_path
BEFORE INSERT OR UPDATE OF parent_id ON categories
FOR EACH ROW
EXECUTE FUNCTION update_category_path();


-- Function to get all descendants of a category
CREATE OR REPLACE FUNCTION get_category_descendants(category_id UUID)
RETURNS TABLE(id UUID, name TEXT, slug TEXT, description TEXT, parent_id UUID, path TEXT, level INTEGER)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    cat_path TEXT;
BEGIN
    -- Get the path of the target category
    SELECT path INTO cat_path
    FROM categories
    WHERE id = get_category_descendants.category_id;
    
    IF NOT FOUND THEN
    RETURN;
    END IF;
    
    -- Return all categories whose path starts with the target category's path
    RETURN QUERY
    SELECT 
        c.id, 
        c.name, 
        c.slug, 
        c.description, 
        c.parent_id, 
        c.path,
        (array_length(string_to_array(c.path, '.'), 1) - array_length(string_to_array(cat_path, '.'), 1)) AS level
    FROM categories c
    WHERE c.path <> cat_path AND c.path LIKE (cat_path || '.%')
    ORDER BY c.path;
END;
$$;

-- Function to get ancestry of a category
CREATE OR REPLACE FUNCTION get_category_ancestry(category_id UUID)
RETURNS TABLE(id UUID, name TEXT, slug TEXT, description TEXT, parent_id UUID, path TEXT, level INTEGER)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    cat_path TEXT;
    path_elements UUID[];
    i INTEGER;
BEGIN
    -- Get the path of the target category
    SELECT path INTO cat_path
    FROM categories
    WHERE id = get_category_ancestry.category_id;
    
    IF NOT FOUND THEN
    RETURN;
    END IF;
    
    -- Convert path to array of UUIDs
    SELECT array_agg(uuid(elem)) INTO path_elements
    FROM unnest(string_to_array(cat_path, '.')) AS elem;
    
    -- Return all categories in the ancestry chain
    FOR i IN 1..array_length(path_elements, 1) LOOP
    RETURN QUERY
    SELECT 
        c.id, 
        c.name, 
        c.slug, 
        c.description, 
        c.parent_id, 
        c.path,
        i - 1 AS level
    FROM categories c
    WHERE c.id = path_elements[i];
    END LOOP;
END;
$$;

-- Function to get full category tree
CREATE OR REPLACE FUNCTION get_category_tree()
RETURNS TABLE(id UUID, name TEXT, slug TEXT, description TEXT, parent_id UUID, path TEXT, level INTEGER)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        c.id, 
        c.name, 
        c.slug, 
        c.description, 
        c.parent_id, 
        c.path,
        array_length(string_to_array(c.path, '.'), 1) - 1 AS level
    FROM categories c
    ORDER BY c.path;
END;
$$;