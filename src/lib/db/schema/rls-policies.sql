-- Enable RLS on all tables
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE product_images ENABLE ROW LEVEL SECURITY;
ALTER TABLE wishlist_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE cart_items ENABLE ROW LEVEL SECURITY;

-- Products Table Policies
CREATE POLICY "Anyone can view products" 
ON products FOR SELECT USING (true);

CREATE POLICY "Sellers can insert products" 
ON products FOR INSERT WITH CHECK (auth.uid() = seller_id);

CREATE POLICY "Sellers can update their products" 
ON products FOR UPDATE USING (auth.uid() = seller_id);

CREATE POLICY "Sellers can delete their products" 
ON products FOR DELETE USING (auth.uid() = seller_id);

-- Categories Table Policies
CREATE POLICY "Anyone can view categories" 
ON categories FOR SELECT USING (true);

CREATE POLICY "Only admins can insert categories" 
ON categories FOR INSERT TO authenticated WITH CHECK (is_admin());

CREATE POLICY "Only admins can update categories" 
ON categories FOR UPDATE TO authenticated USING (is_admin());

CREATE POLICY "Only admins can delete categories" 
ON categories FOR DELETE TO authenticated USING (is_admin());

-- Wishlist Table Policies
CREATE POLICY "Users can view their own wishlist" 
ON wishlist_items FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can add to their own wishlist" 
ON wishlist_items FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can remove from their own wishlist" 
ON wishlist_items FOR DELETE USING (auth.uid() = user_id);

-- Cart Items Table Policies
CREATE POLICY "Users can view their own cart" 
ON cart_items FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can add to their own cart" 
ON cart_items FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own cart" 
ON cart_items FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Users can delete from their own cart" 
ON cart_items FOR DELETE USING (auth.uid() = user_id);