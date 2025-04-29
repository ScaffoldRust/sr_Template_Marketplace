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