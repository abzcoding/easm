#[cfg(test)]
mod tests {
    use backend::models::User;
    use infrastructure::{
        repositories::factory::RepositoryFactory,
        utils::testing::{create_test_organization, setup_test_db},
    };
    use shared::types::UserRole;

    #[tokio::test]
    async fn test_user_repository_basic_operations() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool);
        let user_repo = factory.user_repository();
        let _org_repo = factory.organization_repository();

        // Setup: Create an organization
        let org = create_test_organization(&factory, "Test Org User Repo")
            .await
            .unwrap();

        // 1. Create User
        let new_user = User::new(
            org.id,
            "testuser_crud".to_string(),
            "crud@example.com".to_string(),
            "hashed_password".to_string(),
            Some(UserRole::Analyst),
        );
        let created_user = user_repo
            .create_user(&new_user)
            .await
            .expect("Failed to create user");
        assert_eq!(created_user.username, "testuser_crud");
        assert_eq!(created_user.email, "crud@example.com");
        assert_eq!(created_user.organization_id, org.id);
        assert_eq!(created_user.role, UserRole::Analyst);

        // 2. Get User by ID
        let found_user = user_repo
            .get_user(created_user.id)
            .await
            .expect("Failed to get user by ID");
        assert_eq!(found_user.id, created_user.id);

        // 3. Get User by Username
        let found_by_username = user_repo
            .get_user_by_username("testuser_crud")
            .await
            .expect("Failed to get user by username");
        assert_eq!(found_by_username.unwrap().id, created_user.id);

        // 4. Get User by Email
        let found_by_email = user_repo
            .get_user_by_email("crud@example.com")
            .await
            .expect("Failed to get user by email");
        assert_eq!(found_by_email.unwrap().id, created_user.id);

        // 5. Update User
        let mut user_to_update = found_user.clone();
        user_to_update.role = UserRole::Admin;
        user_to_update.email = "crud_updated@example.com".to_string();
        let updated_user = user_repo
            .update_user(&user_to_update)
            .await
            .expect("Failed to update user");
        assert_eq!(updated_user.id, created_user.id);
        assert_eq!(updated_user.role, UserRole::Admin);
        assert_eq!(updated_user.email, "crud_updated@example.com");

        // Verify update
        let verified_user = user_repo
            .get_user(created_user.id)
            .await
            .expect("Failed to get user after update");
        assert_eq!(verified_user.role, UserRole::Admin);

        // 6. Delete User
        let deleted = user_repo
            .delete_user(created_user.id)
            .await
            .expect("Failed to delete user");
        assert!(deleted);

        // Verify deletion
        let not_found_result = user_repo.get_user(created_user.id).await;
        assert!(not_found_result.is_err());
    }

    #[tokio::test]
    async fn test_user_repository_list_and_filters() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool.clone());
        let user_repo = factory.user_repository();
        let _org_repo = factory.organization_repository();

        // Setup: Create organizations and users
        let org1 = create_test_organization(&factory, "Org 1 Users")
            .await
            .unwrap();
        let org2 = create_test_organization(&factory, "Org 2 Users")
            .await
            .unwrap();

        let _user1_org1_admin = User::new(
            org1.id,
            "user1".into(),
            "user1@org1.com".into(),
            "pwd".into(),
            Some(UserRole::Admin),
        );
        user_repo.create_user(&_user1_org1_admin).await.unwrap();
        let user2_org1_analyst = User::new(
            org1.id,
            "user2".into(),
            "user2@org1.com".into(),
            "pwd".into(),
            Some(UserRole::Analyst),
        );
        user_repo.create_user(&user2_org1_analyst).await.unwrap();
        let _user3_org2_analyst = User::new(
            org2.id,
            "user3".into(),
            "user3@org2.com".into(),
            "pwd".into(),
            Some(UserRole::Analyst),
        );
        user_repo.create_user(&_user3_org2_analyst).await.unwrap();

        // List all for Org 1
        let users_org1 = user_repo
            .list_users(Some(org1.id), None, 10, 0)
            .await
            .expect("Failed to list users for Org 1");
        assert_eq!(users_org1.len(), 2);

        // List Analysts for Org 1
        let analysts_org1 = user_repo
            .list_users(Some(org1.id), Some(UserRole::Analyst), 10, 0)
            .await
            .expect("Failed to list analysts for Org 1");
        assert_eq!(analysts_org1.len(), 1);
        assert_eq!(analysts_org1[0].id, user2_org1_analyst.id);
        assert_eq!(analysts_org1[0].role, UserRole::Analyst);

        // List all users (no org filter)
        let all_users = user_repo
            .list_users(None, None, 10, 0)
            .await
            .expect("Failed to list all users");
        assert_eq!(all_users.len(), 3);

        // Count users for Org 1
        let count_org1 = user_repo
            .count_users(Some(org1.id), None)
            .await
            .expect("Failed to count users for Org 1");
        assert_eq!(count_org1, 2);

        // Count Analysts for Org 1
        let count_analysts_org1 = user_repo
            .count_users(Some(org1.id), Some(UserRole::Analyst))
            .await
            .expect("Failed to count analysts for Org 1");
        assert_eq!(count_analysts_org1, 1);

        // Count all users
        let count_all = user_repo
            .count_users(None, None)
            .await
            .expect("Failed to count all users");
        assert_eq!(count_all, 3);
    }

    #[tokio::test]
    async fn test_user_repository_uniqueness_constraints() {
        let (db_pool, _container) = setup_test_db().await;
        let factory = RepositoryFactory::new(db_pool.clone());
        let user_repo = factory.user_repository();
        let _org_repo = factory.organization_repository();

        // Setup
        let org = create_test_organization(&factory, "Unique User Org")
            .await
            .unwrap();
        let user1 = User::new(
            org.id,
            "unique_user".into(),
            "unique@example.com".into(),
            "pwd".into(),
            None,
        );
        user_repo.create_user(&user1).await.unwrap();

        // Attempt duplicate username
        let duplicate_username_user = User::new(
            org.id,
            "unique_user".into(),
            "other@example.com".into(),
            "pwd".into(),
            None,
        );
        let result_username = user_repo.create_user(&duplicate_username_user).await;
        assert!(
            result_username.is_err(),
            "Should fail on duplicate username"
        );

        // Attempt duplicate email
        let duplicate_email_user = User::new(
            org.id,
            "other_user".into(),
            "unique@example.com".into(),
            "pwd".into(),
            None,
        );
        let result_email = user_repo.create_user(&duplicate_email_user).await;
        assert!(result_email.is_err(), "Should fail on duplicate email");
    }
}
