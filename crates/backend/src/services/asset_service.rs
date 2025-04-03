use async_trait::async_trait;
use shared::types::{AssetStatus, AssetType, ID};
use std::sync::Arc;
use tracing::{debug, info};
use url;

use crate::{
    models::{Asset, AssetRelationshipType},
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

        let cloud_resources: Vec<&Asset> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::CloudResource)
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
                        AssetRelationshipType::Subdomain.as_str().to_string(),
                    ));
                }

                // Check for same registrar relationship (if info available)
                if let (Some(info_a), Some(info_b)) = (
                    domain_a.attributes.get("whois_info"),
                    domain_b.attributes.get("whois_info"),
                ) {
                    if let (Some(registrar_a), Some(registrar_b)) =
                        (info_a.get("registrar"), info_b.get("registrar"))
                    {
                        if registrar_a == registrar_b
                            && registrar_a.as_str().is_some()
                            && !registrar_a.as_str().unwrap().is_empty()
                        {
                            discovered_relationships.push((
                                domain_a.id,
                                domain_b.id,
                                AssetRelationshipType::SameRegistrar.as_str().to_string(),
                            ));
                        }
                    }
                }
            }
        }

        // Discover web app hosted on IP relationships
        for web_app in web_apps.iter() {
            if let Some(host_info) = web_app.attributes.get("host_info") {
                // Check for IP relationship
                if let Some(host_ip) = host_info.get("ip_address") {
                    if let Some(ip_str) = host_ip.as_str() {
                        // Find the corresponding IP asset
                        for ip in ips.iter() {
                            if ip.value == ip_str {
                                discovered_relationships.push((
                                    web_app.id,
                                    ip.id,
                                    AssetRelationshipType::HostedOn.as_str().to_string(),
                                ));
                                break;
                            }
                        }
                    }
                }

                // Check for domain relationship
                if let Some(host_domain) = host_info.get("domain") {
                    if let Some(domain_str) = host_domain.as_str() {
                        // Find the corresponding domain asset
                        for domain in domains.iter() {
                            if domain.value == domain_str
                                || domain_str.ends_with(&format!(".{}", domain.value))
                            {
                                discovered_relationships.push((
                                    web_app.id,
                                    domain.id,
                                    AssetRelationshipType::HostedOn.as_str().to_string(),
                                ));
                                break;
                            }
                        }
                    }
                }
            }

            // Check for web app dependencies
            if let Some(dependencies) = web_app.attributes.get("dependencies") {
                if let Some(deps_array) = dependencies.as_array() {
                    for dep in deps_array {
                        if let Some(dep_url) = dep.get("url").and_then(|u| u.as_str()) {
                            // Try to extract domain from URL
                            if let Ok(url) = url::Url::parse(dep_url) {
                                if let Some(host) = url.host_str() {
                                    // Look for matching domain asset
                                    for domain in domains.iter() {
                                        if domain.value == host
                                            || host.ends_with(&format!(".{}", domain.value))
                                        {
                                            discovered_relationships.push((
                                                web_app.id,
                                                domain.id,
                                                AssetRelationshipType::DependsOn
                                                    .as_str()
                                                    .to_string(),
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
                                            AssetRelationshipType::Secures.as_str().to_string(),
                                        ));
                                    } else if domain_str.ends_with(&format!(".{}", domain.value)) {
                                        // Wildcard certificate that includes this domain
                                        discovered_relationships.push((
                                            cert.id,
                                            domain.id,
                                            AssetRelationshipType::Secures.as_str().to_string(),
                                        ));
                                    }
                                }

                                // Also link to web apps on this domain
                                for web_app in web_apps.iter() {
                                    if let Some(host_info) = web_app.attributes.get("host_info") {
                                        if let Some(app_domain) =
                                            host_info.get("domain").and_then(|d| d.as_str())
                                        {
                                            if app_domain == domain_str
                                                || app_domain.ends_with(&format!(".{}", domain_str))
                                            {
                                                discovered_relationships.push((
                                                    cert.id,
                                                    web_app.id,
                                                    AssetRelationshipType::Secures
                                                        .as_str()
                                                        .to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Check for common issuers between certificates
                if let Some(issuer) = cert_info.get("issuer").and_then(|i| i.as_str()) {
                    for other_cert in certificates.iter() {
                        if cert.id == other_cert.id {
                            continue;
                        }

                        if let Some(other_info) = other_cert.attributes.get("certificate_info") {
                            if let Some(other_issuer) =
                                other_info.get("issuer").and_then(|i| i.as_str())
                            {
                                if issuer == other_issuer {
                                    discovered_relationships.push((
                                        cert.id,
                                        other_cert.id,
                                        AssetRelationshipType::SameIssuer.as_str().to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Discover cloud resource relationships
        for cloud_resource in cloud_resources.iter() {
            if let Some(cloud_info) = cloud_resource.attributes.get("cloud_info") {
                // Check for resources in the same VPC/Network
                if let Some(vpc_id) = cloud_info.get("vpc_id").and_then(|v| v.as_str()) {
                    for other_resource in cloud_resources.iter() {
                        if cloud_resource.id == other_resource.id {
                            continue;
                        }

                        if let Some(other_info) = other_resource.attributes.get("cloud_info") {
                            if let Some(other_vpc) =
                                other_info.get("vpc_id").and_then(|v| v.as_str())
                            {
                                if vpc_id == other_vpc {
                                    discovered_relationships.push((
                                        cloud_resource.id,
                                        other_resource.id,
                                        AssetRelationshipType::SameNetwork.as_str().to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }

                // Link cloud resources to their public IP addresses
                if let Some(public_ip) = cloud_info.get("public_ip").and_then(|ip| ip.as_str()) {
                    for ip in ips.iter() {
                        if ip.value == public_ip {
                            discovered_relationships.push((
                                cloud_resource.id,
                                ip.id,
                                AssetRelationshipType::HasPublicIP.as_str().to_string(),
                            ));
                            break;
                        }
                    }
                }

                // Link cloud resources to hosted services/web apps
                if let Some(resource_dns) = cloud_info.get("dns_name").and_then(|dns| dns.as_str())
                {
                    for web_app in web_apps.iter() {
                        if let Some(host_info) = web_app.attributes.get("host_info") {
                            if let Some(app_host) =
                                host_info.get("hostname").and_then(|h| h.as_str())
                            {
                                if app_host == resource_dns {
                                    discovered_relationships.push((
                                        web_app.id,
                                        cloud_resource.id,
                                        AssetRelationshipType::HostedOn.as_str().to_string(),
                                    ));
                                }
                            }
                        }
                    }

                    // Also link to domains
                    for domain in domains.iter() {
                        if resource_dns.ends_with(&format!(".{}", domain.value)) {
                            discovered_relationships.push((
                                cloud_resource.id,
                                domain.id,
                                AssetRelationshipType::BelongsTo.as_str().to_string(),
                            ));
                        }
                    }
                }
            }
        }

        // Deduplicate relationships
        let mut unique_relationships = Vec::new();
        for rel in discovered_relationships {
            if !unique_relationships.contains(&rel) {
                unique_relationships.push(rel);
            }
        }

        debug!(
            "Discovered {} unique asset relationships",
            unique_relationships.len()
        );
        Ok(unique_relationships)
    }

    // Helper function to identify direct and indirect dependencies between assets
    async fn analyze_dependency_chain(
        &self,
        asset_id: ID,
        max_depth: usize,
    ) -> Result<Vec<Vec<ID>>> {
        debug!(
            "Analyzing dependency chains for asset: {} (max depth: {})",
            asset_id, max_depth
        );

        let mut dependency_chains = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Start DFS from the asset
        self.find_dependency_paths(
            asset_id,
            Vec::new(),
            &mut dependency_chains,
            &mut visited,
            max_depth,
        )
        .await?;

        debug!("Found {} dependency chains", dependency_chains.len());
        Ok(dependency_chains)
    }

    // Helper method that implements depth-first search to find dependency paths
    async fn find_dependency_paths(
        &self,
        current_id: ID,
        current_path: Vec<ID>,
        all_paths: &mut Vec<Vec<ID>>,
        visited: &mut std::collections::HashSet<ID>,
        max_depth: usize,
    ) -> Result<()> {
        // Check if we've visited this node in the current path
        if visited.contains(&current_id) {
            return Ok(());
        }

        // Add the current asset to the path
        let mut path = current_path.clone();
        path.push(current_id);

        // Mark as visited
        visited.insert(current_id);

        // Get related assets (specifically dependencies)
        let related = self
            .get_related_assets(
                current_id,
                Some(AssetRelationshipType::DependsOn.as_str().to_string()),
            )
            .await?;

        if related.is_empty() || path.len() >= max_depth {
            // End of a path, add it to results
            all_paths.push(path.clone());
        } else {
            // Continue the search for each dependency
            for (asset, _) in related {
                self.find_dependency_paths(asset.id, path.clone(), all_paths, visited, max_depth)
                    .await?;
            }
        }

        // Remove from visited on backtrack
        visited.remove(&current_id);

        Ok(())
    }
}
