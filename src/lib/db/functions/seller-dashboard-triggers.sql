-- Triggers to automatically update store metrics

-- Trigger for orders table
DROP TRIGGER IF EXISTS trigger_update_store_metrics_on_order ON orders;
CREATE TRIGGER trigger_update_store_metrics_on_order
    AFTER INSERT OR UPDATE OR DELETE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION update_store_metrics();

-- Trigger for products table
DROP TRIGGER IF EXISTS trigger_update_store_metrics_on_product ON products;
CREATE TRIGGER trigger_update_store_metrics_on_product
    AFTER INSERT OR UPDATE OR DELETE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_store_metrics();

-- Trigger for reviews table (if it exists)
DROP TRIGGER IF EXISTS trigger_update_store_metrics_on_review ON reviews;
CREATE TRIGGER trigger_update_store_metrics_on_review
    AFTER INSERT OR UPDATE OR DELETE ON reviews
    FOR EACH ROW
    EXECUTE FUNCTION update_store_metrics();

-- Trigger to track order status changes
DROP TRIGGER IF EXISTS trigger_track_order_status_change ON orders;
CREATE TRIGGER trigger_track_order_status_change
    AFTER UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION track_order_status_change();

-- Trigger to automatically calculate daily analytics when orders are created
DROP TRIGGER IF EXISTS trigger_calculate_daily_analytics ON orders;
CREATE TRIGGER trigger_calculate_daily_analytics
    AFTER INSERT OR UPDATE ON orders
    FOR EACH ROW
    EXECUTE FUNCTION calculate_daily_analytics(NEW.store_id, DATE(NEW.created_at)); 