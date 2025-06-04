-- Products Table
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    description TEXT,
    price DECIMAL(12, 2) NOT NULL CHECK (price > 0),
    category UUID REFERENCES categories(id),
    rating DECIMAL(3, 2) DEFAULT 0 CHECK (rating >= 0 AND rating <= 5),
    rating_count INTEGER DEFAULT 0 CHECK (rating_count >= 0),
    seller_id UUID REFERENCES auth.users(id),
    store_id UUID REFERENCES stores(id),
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    stock INTEGER DEFAULT 0 CHECK (stock >= 0),
    slug TEXT UNIQUE NOT NULL,
    featured BOOLEAN DEFAULT false,
    variant TEXT,
    name TEXT -- Alternative to title for consistency
);