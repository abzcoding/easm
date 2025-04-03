#[cfg(test)]
mod tests {
    use backend::models::Organization;
    use infrastructure::{repositories::factory::RepositoryFactory, utils::testing::setup_test_db};

    #[tokio::test]
    async fn test_organization_repository_basic_operations() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let org_repo = factory.organization_repository();

        // 1. Create Organization
        let new_org = Organization::new("Test Org CRUD".to_string());
        let created_org = org_repo
            .create_organization(&new_org)
            .await
            .expect("Failed to create organization");
        assert_eq!(created_org.name, "Test Org CRUD");
        let org_id = created_org.id;

        // 2. Get Organization
        let found_org = org_repo
            .get_organization(org_id)
            .await
            .expect("Failed to get organization");
        assert_eq!(found_org.id, org_id);
        assert_eq!(found_org.name, "Test Org CRUD");

        // 3. Update Organization
        let mut org_to_update = found_org.clone();
        org_to_update.name = "Test Org CRUD Updated".to_string();
        let updated_org = org_repo
            .update_organization(&org_to_update)
            .await
            .expect("Failed to update organization");
        assert_eq!(updated_org.id, org_id);
        assert_eq!(updated_org.name, "Test Org CRUD Updated");

        // Verify update
        let verified_org = org_repo
            .get_organization(org_id)
            .await
            .expect("Failed to get organization after update");
        assert_eq!(verified_org.name, "Test Org CRUD Updated");

        // 4. Delete Organization
        let deleted = org_repo
            .delete_organization(org_id)
            .await
            .expect("Failed to delete organization");
        assert!(deleted);

        // Verify deletion
        let not_found_result = org_repo.get_organization(org_id).await;
        assert!(not_found_result.is_err());
    }

    #[tokio::test]
    async fn test_organization_repository_list_and_count() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let org_repo = factory.organization_repository();

        // Initial count
        let initial_count = org_repo
            .count_organizations()
            .await
            .expect("Failed to count initial orgs");

        // Create some organizations
        let org1 = Organization::new("Org List 1".to_string());
        org_repo.create_organization(&org1).await.unwrap();
        let org2 = Organization::new("Org List 2".to_string());
        org_repo.create_organization(&org2).await.unwrap();
        let org3 = Organization::new("Org List 3".to_string());
        org_repo.create_organization(&org3).await.unwrap();

        // Count after creation
        let count_after_create = org_repo
            .count_organizations()
            .await
            .expect("Failed to count orgs after create");
        assert_eq!(count_after_create, initial_count + 3);

        // List with limit and offset
        let orgs_page1 = org_repo
            .list_organizations(2, initial_count + 0)
            .await
            .expect("Failed to list orgs page 1");
        assert_eq!(orgs_page1.len(), 2);
        // Names might be org1, org2 depending on default DB order

        let orgs_page2 = org_repo
            .list_organizations(2, initial_count + 2)
            .await
            .expect("Failed to list orgs page 2");
        assert_eq!(orgs_page2.len(), 1); // Only org3 left
        assert_eq!(orgs_page2[0].name, "Org List 3");

        // List with limit larger than remaining items
        let orgs_page_large = org_repo
            .list_organizations(10, initial_count + 0)
            .await
            .expect("Failed to list orgs large page");
        assert_eq!(orgs_page_large.len(), 3);
    }

    #[tokio::test]
    async fn test_organization_repository_uniqueness_constraints() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let org_repo = factory.organization_repository();

        // Setup
        let org1 = Organization::new("Unique Org Name".to_string());
        org_repo.create_organization(&org1).await.unwrap();

        // Attempt duplicate name
        let duplicate_org = Organization::new("Unique Org Name".to_string());
        let result = org_repo.create_organization(&duplicate_org).await;
        assert!(
            result.is_err(),
            "Should fail on duplicate organization name"
        );
    }
}
