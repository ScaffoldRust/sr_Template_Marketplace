-- Indexes for performance
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_price ON products(price);
CREATE INDEX idx_products_rating ON products(rating);
CREATE INDEX idx_products_created_at ON products(created_at);
CREATE INDEX idx_categories_slug ON categories(slug);
CREATE INDEX idx_wishlist_user_id ON wishlist_items(user_id);
CREATE INDEX idx_cart_user_id ON cart_items(user_id);