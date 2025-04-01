#[cfg(test)]
mod tests {
    use backend::models::Asset;
    use shared::types::{AssetStatus, AssetType};
    use uuid::Uuid;
    #[test]
    fn test_asset_new() {
        let org_id = Uuid::new_v4();
        let asset_type = AssetType::Domain;
        let value = "example.com".to_string();

        let asset = Asset::new(org_id, asset_type, value.clone(), None);

        assert_eq!(asset.organization_id, org_id);
        assert_eq!(asset.asset_type, asset_type);
        assert_eq!(asset.value, value);
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
        let asset_type = AssetType::IPAddress;
        let value = "192.0.2.1".to_string();

        let attributes = serde_json::json!({
            "location": "us-east-1",
            "tags": ["production", "web"]
        });

        let asset = Asset::new(org_id, asset_type, value, Some(attributes.clone()));

        assert_eq!(asset.organization_id, org_id);
        assert_eq!(asset.asset_type, asset_type);
        assert_eq!(asset.attributes, attributes);
    }
}
