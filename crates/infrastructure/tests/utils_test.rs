use chrono::{TimeZone, Utc};
use infrastructure::utils::{from_offset_datetime, to_offset_datetime};

#[test]
fn test_datetime_conversion() {
    // Create a specific UTC datetime
    let dt_utc = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

    // Convert to database offset datetime
    let db_dt = to_offset_datetime(dt_utc);

    // Convert back from offset datetime
    let round_trip = from_offset_datetime(Some(db_dt));

    // Verify round trip results in same time
    assert_eq!(round_trip, dt_utc);
}

#[test]
fn test_datetime_conversion_with_none() {
    // Test None value is handled correctly
    let result = from_offset_datetime(None);

    // Should be current time
    let now = Utc::now();

    // Check that returned time is within 1 second of now
    let diff = (result - now).num_milliseconds().abs();
    assert!(diff < 1000, "Time difference too large: {}ms", diff);
}

#[test]
fn test_datetime_precision() {
    // Create a datetime with microsecond precision
    let dt = Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap();

    // Convert to database offset datetime
    let db_dt = to_offset_datetime(dt);

    // Convert back
    let round_trip = from_offset_datetime(Some(db_dt));

    // Verify timestamps match
    assert_eq!(round_trip.timestamp(), dt.timestamp());
    assert_eq!(round_trip, dt);
}

// Test the repository testing utilities
#[cfg(test)]
mod repository_testing_utils {
    use backend::Result;
    use infrastructure::{
        database::migrations::Migrator,
        repositories::RepositoryFactory,
        utils::testing::{create_test_asset, create_test_organization, create_test_user},
    };
    use shared::types::{AssetType, UserRole};
    use sqlx::PgPool;

    // Helper function to run migrations before tests
    async fn setup_database(pool: &PgPool) -> Result<()> {
        let migrator = Migrator::new(pool.clone());
        migrator.run_migrations().await.unwrap();
        Ok(())
    }

    #[sqlx::test]
    async fn test_create_test_organization(pool: PgPool) -> Result<()> {
        // Run migrations first
        setup_database(&pool).await?;

        let factory = RepositoryFactory::new(pool);

        // Create test organization
        let org = create_test_organization(&factory, "Test Organization").await?;

        // Verify
        assert_eq!(org.name, "Test Organization");

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_test_user(pool: PgPool) -> Result<()> {
        // Run migrations first
        setup_database(&pool).await?;

        let factory = RepositoryFactory::new(pool);

        // Create test organization
        let org = create_test_organization(&factory, "Test Organization").await?;

        // Create test user
        let user = create_test_user(
            &factory,
            org.id,
            "testuser",
            "test@example.com",
            UserRole::Admin,
        )
        .await?;

        // Verify
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Admin);

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_test_asset(pool: PgPool) -> Result<()> {
        // Run migrations first
        setup_database(&pool).await?;

        let factory = RepositoryFactory::new(pool);

        // Create test organization
        let org = create_test_organization(&factory, "Test Organization").await?;

        // Create test asset
        let asset = create_test_asset(&factory, org.id, AssetType::Domain, "example.com").await?;

        // Verify
        assert_eq!(asset.value, "example.com");
        assert_eq!(asset.asset_type, AssetType::Domain);

        Ok(())
    }
}
