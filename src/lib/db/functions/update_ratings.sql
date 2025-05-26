-- Check if reviews table exists, create it if not
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_name = 'reviews'
    ) THEN
        CREATE TABLE reviews (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            product_id UUID REFERENCES products(id) ON DELETE CASCADE,
            user_id UUID REFERENCES auth.users(id),
            rating INTEGER CHECK (rating >= 1 AND rating <= 5),
            comment TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(product_id, user_id)
        );

        -- Enable RLS
        ALTER TABLE reviews ENABLE ROW LEVEL SECURITY;

        CREATE POLICY "Anyone can view reviews" 
            ON reviews FOR SELECT USING (true);

        CREATE POLICY "Users can add reviews" 
            ON reviews FOR INSERT WITH CHECK (auth.uid() = user_id);

        CREATE POLICY "Users can update their own reviews" 
            ON reviews FOR UPDATE USING (auth.uid() = user_id);

        CREATE POLICY "Users can delete their own reviews" 
            ON reviews FOR DELETE USING (auth.uid() = user_id);
    END IF;
END
$$;

-- Function to update product rating
CREATE OR REPLACE FUNCTION update_product_rating()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate new rating for the product
    UPDATE products
    SET rating = (
        SELECT AVG(rating)::DECIMAL(3,2)
        FROM reviews
        WHERE product_id = CASE
            WHEN TG_OP = 'DELETE' THEN OLD.product_id
            ELSE NEW.product_id
        END
    ),
    rating_count = (
        SELECT COUNT(*)
        FROM reviews
        WHERE product_id = CASE
            WHEN TG_OP = 'DELETE' THEN OLD.product_id
            ELSE NEW.product_id
        END
    )
    WHERE id = CASE
        WHEN TG_OP = 'DELETE' THEN OLD.product_id
        ELSE NEW.product_id
    END;
    
    RETURN CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_product_rating_trigger ON reviews;
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
$$;
