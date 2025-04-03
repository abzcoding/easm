#[cfg(test)]
mod tests {
    use backend::models::Asset;
    use shared::types::{AssetStatus, AssetType};
    use uuid::Uuid;
    #[test]
    fn test_asset_new() {
        let org_id = Uuid::new_v4();
        let asset = Asset::new(org_id, AssetType::Domain, "example.com".to_string(), None);

        assert_eq!(asset.organization_id, org_id);
        assert_eq!(asset.asset_type, AssetType::Domain);
        assert_eq!(asset.value, "example.com");
        assert_eq!(asset.status, AssetStatus::Active);

        // Verify timestamps were set
        let now = chrono::Utc::now();
        assert!(asset.created_at <= now);
        assert!(asset.updated_at <= now);
        assert!(asset.first_seen <= now);
        assert!(asset.last_seen <= now);

        // Verify attributes default
        match asset.attributes {
            serde_json::Value::Object(map) => assert!(map.is_empty()),
            _ => panic!("Expected empty object for default attributes"),
        }
    }

    #[test]
    fn test_asset_with_attributes() {
        let org_id = Uuid::new_v4();
        let attrs = serde_json::json!({
            "dns_provider": "Example DNS",
            "registrar": "Example Registrar"
        });

        let asset = Asset::new(
            org_id,
            AssetType::Domain,
            "example.com".to_string(),
            Some(attrs.clone()),
        );

        assert_eq!(asset.organization_id, org_id);
        assert_eq!(asset.asset_type, AssetType::Domain);
        assert_eq!(asset.value, "example.com");
        assert_eq!(asset.attributes, attrs);
    }
    
    #[test]
    fn test_asset_builder() {
        let org_id = Uuid::new_v4();
        let attrs = serde_json::json!({
            "dns_provider": "Example DNS",
            "registrar": "Example Registrar"
        });
        
        // Test the builder with various settings
        let asset = Asset::builder(org_id, AssetType::Domain, "example.com".to_string())
            .status(AssetStatus::Inactive)
            .attributes(attrs.clone())
            .build();
            
        assert_eq!(asset.organization_id, org_id);
        assert_eq!(asset.asset_type, AssetType::Domain);
        assert_eq!(asset.value, "example.com");
        assert_eq!(asset.status, AssetStatus::Inactive); // Custom status
        assert_eq!(asset.attributes, attrs);
        assert_eq!(asset.first_seen, asset.last_seen); // Should be the same by default
    }
}
