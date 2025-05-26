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

-- Create database trigger to maintain category paths for hierarchical categories
CREATE OR REPLACE FUNCTION update_category_path()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    parent_path TEXT;
BEGIN
    IF NEW.parent_id IS NULL THEN
        NEW.path = NEW.id::TEXT;
    ELSE
        SELECT path INTO parent_path
        FROM categories
        WHERE id = NEW.parent_id;
        
        NEW.path = parent_path || '.' || NEW.id::TEXT;
    END IF;
    
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trigger_update_category_path ON categories;
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
