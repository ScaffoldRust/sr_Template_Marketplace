-- function to generate a unique slug
CREATE OR REPLACE FUNCTION generate_unique_slug(input_title TEXT, table_name TEXT, existing_id UUID DEFAULT NULL)
RETURNS TEXT AS $$
DECLARE
    base_slug TEXT;
    new_slug TEXT;
    counter INTEGER := 1;
    slug_exists BOOLEAN;
    query TEXT;
BEGIN
    base_slug := lower(regexp_replace(input_title, '[^a-zA-Z0-9\s]', '', 'g'));
    base_slug := regexp_replace(base_slug, '\s+', '-', 'g');
    
    new_slug := base_slug;
    
    LOOP
    query := format('SELECT EXISTS(SELECT 1 FROM %I WHERE slug = $1 AND ($2 IS NULL OR id != $2))', table_name);
    EXECUTE query INTO slug_exists USING new_slug, existing_id;
    
    EXIT WHEN NOT slug_exists;
    
    counter := counter + 1;
    new_slug := base_slug || '-' || counter;
    END LOOP;
    
    RETURN new_slug;
END;
$$ LANGUAGE plpgsql;

-- Create slugify function to generate slugs from titles
CREATE OR REPLACE FUNCTION slugify(input_text TEXT)
RETURNS TEXT
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
    result TEXT;
BEGIN
    result := lower(input_text);
    
    result := regexp_replace(result, '[^a-z0-9]', '-', 'g');
    
    result := regexp_replace(result, '-+', '-', 'g');
    
    result := trim(both '-' from result);

    RETURN result;
END;
$$;


-- Create a function to automatically slugify products
CREATE OR REPLACE FUNCTION auto_slugify_product()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    base_slug TEXT;
    new_slug TEXT;
    counter INTEGER := 1;
    slug_exists BOOLEAN;
BEGIN
    IF TG_OP = 'UPDATE' AND NEW.slug = OLD.slug THEN
        RETURN NEW;
    END IF;
    
    IF NEW.slug IS NULL OR NEW.slug = '' THEN
    base_slug := slugify(NEW.title);
    
    new_slug := base_slug;
    LOOP
        SELECT EXISTS(
            SELECT 1 FROM products 
            WHERE slug = new_slug 
            AND id != NEW.id
        ) INTO slug_exists;
        
        EXIT WHEN NOT slug_exists;
        
        counter := counter + 1;
        new_slug := base_slug || '-' || counter;
        END LOOP;
        
        NEW.slug := new_slug;
    END IF;
    
    RETURN NEW;
END;
$$;

CREATE TRIGGER trigger_auto_slugify_product
BEFORE INSERT OR UPDATE ON products
FOR EACH ROW
EXECUTE FUNCTION auto_slugify_product();


CREATE OR REPLACE FUNCTION auto_slugify_category()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    base_slug TEXT;
    new_slug TEXT;
    counter INTEGER := 1;
    slug_exists BOOLEAN;
BEGIN
    IF TG_OP = 'UPDATE' AND NEW.slug = OLD.slug THEN
        RETURN NEW;
    END IF;
    
    -- If no slug is provided, generate one from the name
    IF NEW.slug IS NULL OR NEW.slug = '' THEN
    base_slug := slugify(NEW.name);
    
    new_slug := base_slug;
    LOOP
        SELECT EXISTS(
            SELECT 1 FROM categories 
            WHERE slug = new_slug 
            AND id != NEW.id
        ) INTO slug_exists;
        
        EXIT WHEN NOT slug_exists;
        
        counter := counter + 1;
        new_slug := base_slug || '-' || counter;
        END LOOP;
        
        NEW.slug := new_slug;
    END IF;
    
    RETURN NEW;
END;
$$;

-- Create a trigger to automatically slugify categories
CREATE TRIGGER trigger_auto_slugify_category
BEFORE INSERT OR UPDATE ON categories
FOR EACH ROW
EXECUTE FUNCTION auto_slugify_category();