-- Create an update_stock function to handle stock management
CREATE OR REPLACE FUNCTION update_stock(product_id UUID, quantity INTEGER)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    current_stock INTEGER;
    BEGIN
    SELECT stock INTO current_stock
    FROM products
    WHERE id = update_stock.product_id;
    
    IF current_stock < update_stock.quantity THEN
        RETURN false;
    END IF;
    
    UPDATE products
    SET stock = stock - update_stock.quantity
    WHERE id = update_stock.product_id;
    
    RETURN true;
END;
$$;

-- Create a function to check if a product is in stock
CREATE OR REPLACE FUNCTION is_product_in_stock(
    product_id UUID,
    quantity INTEGER DEFAULT 1
)
RETURNS BOOLEAN
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    available_stock INTEGER;
BEGIN
    SELECT stock INTO available_stock
    FROM products
    WHERE id = is_product_in_stock.product_id;
    
    RETURN FOUND AND available_stock >= is_product_in_stock.quantity;
END;
$$;

-- Create a function to validate cart items against stock
CREATE OR REPLACE FUNCTION validate_cart_stock(user_id UUID)
RETURNS TABLE(
    valid BOOLEAN,
    invalid_items JSON
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    invalid_items_array JSON;
BEGIN
    -- Find cart items that exceed available stock
    SELECT json_agg(
    json_build_object(
        'product_id', c.product_id,
        'title', p.title,
        'requested', c.quantity,
        'available', p.stock
    )
    ) INTO invalid_items_array
    FROM cart_items c
    JOIN products p ON c.product_id = p.id
    WHERE c.user_id = validate_cart_stock.user_id
    AND c.quantity > p.stock;
    
    RETURN QUERY
    SELECT
    (invalid_items_array IS NULL OR json_array_length(invalid_items_array) = 0) AS valid,
    COALESCE(invalid_items_array, '[]'::JSON) AS invalid_items;
END;
$$;