-- Function to create and trigger to update product rating
CREATE OR REPLACE FUNCTION update_product_rating()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate new rating for the product
    UPDATE products
    SET rating = (
    SELECT AVG(rating)
    FROM reviews
    WHERE product_id = NEW.product_id
    ),
    rating_count = (
        SELECT COUNT(*)
        FROM reviews
        WHERE product_id = NEW.product_id
    )
    WHERE id = NEW.product_id;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update product rating when reviews change
CREATE TRIGGER update_product_rating_trigger
AFTER INSERT OR UPDATE OR DELETE ON reviews
FOR EACH ROW
EXECUTE FUNCTION update_product_rating();

-- Create a function to recalculate product ratings
CREATE OR REPLACE FUNCTION recalculate_product_rating(product_id UUID)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    UPDATE products
    SET 
        rating = COALESCE((
        SELECT AVG(rating)::DECIMAL(3,2)
        FROM reviews
        WHERE reviews.product_id = recalculate_product_rating.product_id
        ), 0),
        rating_count = COALESCE((
        SELECT COUNT(*)
        FROM reviews
        WHERE reviews.product_id = recalculate_product_rating.product_id
        ), 0)
    WHERE id = recalculate_product_rating.product_id;
END;
$$