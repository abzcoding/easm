use async_trait::async_trait;
use shared::types::{AssetStatus, AssetType, ID};
use std::sync::Arc;
use tracing::{debug, info};

use crate::{
    models::{Asset, AssetRelationship, AssetRelationshipType},
    traits::{AssetRepository, AssetService},
    Result,
};

pub struct AssetServiceImpl {
    repository: Arc<dyn AssetRepository>,
}

impl AssetServiceImpl {
    pub fn new(repository: Arc<dyn AssetRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl AssetService for AssetServiceImpl {
    async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
        debug!("Creating asset: {}", asset.value);
        self.repository.create_asset(asset).await
    }

    async fn get_asset(&self, id: ID) -> Result<Asset> {
        debug!("Getting asset with id: {}", id);
        self.repository.get_asset(id).await
    }

    async fn update_asset(&self, asset: &Asset) -> Result<Asset> {
        debug!("Updating asset: {}", asset.value);
        self.repository.update_asset(asset).await
    }

    async fn delete_asset(&self, id: ID) -> Result<bool> {
        debug!("Deleting asset with id: {}", id);
        self.repository.delete_asset(id).await
    }

    async fn list_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Asset>> {
        debug!(
            "Listing assets with filters - organization_id: {:?}, asset_type: {:?}, status: {:?}, limit: {}, offset: {}",
            organization_id, asset_type, status, limit, offset
        );
        self.repository
            .list_assets(organization_id, asset_type, status, limit, offset)
            .await
    }

    async fn count_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
    ) -> Result<usize> {
        debug!(
            "Counting assets with filters - organization_id: {:?}, asset_type: {:?}, status: {:?}",
            organization_id, asset_type, status
        );
        self.repository
            .count_assets(organization_id, asset_type, status)
            .await
    }

    async fn create_asset_relationship(
        &self,
        source_asset_id: ID,
        target_asset_id: ID,
        relationship_type: String,
        _metadata: Option<serde_json::Value>,
    ) -> Result<bool> {
        info!(
            "Creating relationship: {} between assets {} and {}",
            relationship_type, source_asset_id, target_asset_id
        );

        // Get the source asset
        let mut source_asset = self.repository.get_asset(source_asset_id).await?;

        // Add the relationship to the source asset
        source_asset.add_relationship(&relationship_type, target_asset_id);

        // Update the source asset
        self.repository.update_asset(&source_asset).await?;

        Ok(true)
    }

    async fn delete_asset_relationship(
        &self,
        source_asset_id: ID,
        target_asset_id: ID,
        relationship_type: String,
    ) -> Result<bool> {
        info!(
            "Deleting relationship: {} between assets {} and {}",
            relationship_type, source_asset_id, target_asset_id
        );

        // Get the source asset
        let mut source_asset = self.repository.get_asset(source_asset_id).await?;

        // Remove the relationship from the source asset
        source_asset.remove_relationship(&relationship_type, target_asset_id);

        // Update the source asset
        self.repository.update_asset(&source_asset).await?;

        Ok(true)
    }

    async fn get_related_assets(
        &self,
        asset_id: ID,
        relationship_type: Option<String>,
    ) -> Result<Vec<(Asset, String)>> {
        debug!(
            "Getting related assets for asset id: {} with relationship type: {:?}",
            asset_id, relationship_type
        );

        // Get the source asset
        let source_asset = self.repository.get_asset(asset_id).await?;

        // Get the relationships from the asset
        let relationships = source_asset.get_relationships();

        // Filter by relationship type if specified
        let filtered_relationships = if let Some(rel_type) = relationship_type {
            relationships
                .iter()
                .filter(|(k, _)| **k == rel_type)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<_, _>>()
        } else {
            relationships.clone()
        };

        // Fetch the related assets
        let mut related_assets = Vec::new();
        for (rel_type, asset_ids) in filtered_relationships {
            for target_id in asset_ids {
                if let Ok(asset) = self.repository.get_asset(target_id).await {
                    related_assets.push((asset, rel_type.clone()));
                }
            }
        }

        Ok(related_assets)
    }

    async fn discover_asset_relationships(
        &self,
        organization_id: ID,
    ) -> Result<Vec<(ID, ID, String)>> {
        info!(
            "Discovering asset relationships for organization: {}",
            organization_id
        );

        // Get all assets for the organization
        let assets = self
            .repository
            .list_assets(Some(organization_id), None, None, 1000, 0)
            .await?;

        let mut discovered_relationships = Vec::new();

        // Group assets by type for easier processing
        let domains: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::Domain)
            .collect();

        let ips: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::IPAddress)
            .collect();

        let web_apps: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::WebApp)
            .collect();

        let certificates: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::Certificate)
            .collect();

        // Discover domain relationships (e.g., subdomains)
        for domain_a in domains.iter() {
            for domain_b in domains.iter() {
                if domain_a.id == domain_b.id {
                    continue;
                }

                // Check for subdomain relationship
                if domain_b.value.ends_with(&format!(".{}", domain_a.value)) {
                    discovered_relationships.push((
                        domain_b.id,
                        domain_a.id,
                        AssetRelationshipType::Subdomain.as_str(),
                    ));
                }
            }
        }

        // Discover web app hosted on IP relationships
        for web_app in web_apps.iter() {
            if let Some(host_info) = web_app.attributes.get("host_info") {
                if let Some(host_ip) = host_info.get("ip_address") {
                    if let Some(ip_str) = host_ip.as_str() {
                        // Find the corresponding IP asset
                        for ip in ips.iter() {
                            if ip.value == ip_str {
                                discovered_relationships.push((
                                    web_app.id,
                                    ip.id,
                                    AssetRelationshipType::HostedOn.as_str(),
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Discover certificate relationships
        for cert in certificates.iter() {
            if let Some(cert_info) = cert.attributes.get("certificate_info") {
                if let Some(cert_domains) = cert_info.get("domains") {
                    if let Some(domains_array) = cert_domains.as_array() {
                        for domain_value in domains_array {
                            if let Some(domain_str) = domain_value.as_str() {
                                // Find the corresponding domain asset
                                for domain in domains.iter() {
                                    if domain.value == domain_str {
                                        discovered_relationships.push((
                                            cert.id,
                                            domain.id,
                                            AssetRelationshipType::CertificateFor.as_str(),
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        info!(
            "Discovered {} potential relationships",
            discovered_relationships.len()
        );
        Ok(discovered_relationships)
    }
}
