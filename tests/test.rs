#[cfg(test)]
mod tests {
    use Database::SledClient;

    use super::*;

    const TEST_URL: &str = "http://localhost:3030";

    #[tokio::test]
    async fn test_health_check() {
        let client = SledClient::new(TEST_URL);
        let healthy = client.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_tree_operations() {
        let client = SledClient::new(TEST_URL);
        
        // Test inserción
        let result = client.tree_insert("users", "user1", "John Doe").await;
        assert!(result.is_ok());
        
        // Test obtención
        let value = client.tree_get("users", "user1").await.unwrap();
        assert_eq!(value, Some("John Doe".to_string()));
        
        // Test eliminación
        let result = client.tree_delete("users", "user1").await;
        assert!(result.is_ok());
        
        // Verificar que fue eliminado
        let value = client.tree_get("users", "user1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_tree_listing() {
        let client = SledClient::new(TEST_URL);
        
        // Insertar algunos datos
        client.tree_insert("products", "prod1", "Laptop").await.unwrap();
        client.tree_insert("products", "prod2", "Mouse").await.unwrap();
        
        // Listar claves
        let keys = client.tree_list_keys("products").await.unwrap();
        assert!(keys.contains(&"prod1".to_string()));
        assert!(keys.contains(&"prod2".to_string()));
        
        // Limpiar
        client.tree_delete("products", "prod1").await.unwrap();
        client.tree_delete("products", "prod2").await.unwrap();
    }

    #[tokio::test]
    async fn test_legacy_operations() {
        let client = SledClient::new(TEST_URL);
        
        // Test inserción legacy
        let result = client.insert("legacy_key", "legacy_value").await;
        assert!(result.is_ok());
        
        // Test obtención legacy
        let value = client.get("legacy_key").await.unwrap();
        assert_eq!(value, Some("legacy_value".to_string()));
    }
}