use backend::{models::Port, Result};
use infrastructure::{
    database::migrations::Migrator,
    repositories::RepositoryFactory,
    utils::testing::{create_test_asset, create_test_organization},
};
use shared::types::{AssetType, PortStatus, Protocol};
use sqlx::PgPool;

// Helper function to run migrations before tests
async fn setup_database(pool: &PgPool) -> Result<()> {
    let migrator = Migrator::new(pool.clone());
    migrator.run_migrations().await.unwrap();
    Ok(())
}

#[sqlx::test]
async fn test_port_repository_basic_operations(pool: PgPool) -> Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let port_repo = factory.port_repository();

    // Create test organization and asset
    let org = create_test_organization(&factory, "Test Organization").await?;
    let asset = create_test_asset(&factory, org.id, AssetType::IPAddress, "192.168.1.1").await?;

    // Create port
    let port = Port::new(asset.id, 80, Protocol::TCP, Some("http".to_string()), None);

    // Test create
    let created_port = port_repo.create_port(&port).await?;
    assert_eq!(created_port.port_number, 80);
    assert_eq!(created_port.protocol, Protocol::TCP);
    assert_eq!(created_port.service_name, Some("http".to_string()));
    assert_eq!(created_port.status, PortStatus::Open);

    // Test get
    let fetched_port = port_repo.get_port(created_port.id).await?;
    assert_eq!(fetched_port.id, created_port.id);
    assert_eq!(fetched_port.port_number, created_port.port_number);

    // Test update
    let mut port_to_update = fetched_port.clone();
    port_to_update.service_name = Some("https".to_string());
    port_to_update.banner = Some("Apache/2.4.41".to_string());

    let updated_port = port_repo.update_port(&port_to_update).await?;
    assert_eq!(updated_port.service_name, Some("https".to_string()));
    assert_eq!(updated_port.banner, Some("Apache/2.4.41".to_string()));

    // Test list
    let ports = port_repo
        .list_ports(Some(asset.id), None, None, None, 10, 0)
        .await?;

    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].id, created_port.id);

    // Test count
    let count = port_repo
        .count_ports(Some(asset.id), None, None, None)
        .await?;

    assert_eq!(count, 1);

    // Test delete
    let deleted = port_repo.delete_port(created_port.id).await?;
    assert!(deleted);

    // Verify deleted
    let count_after_delete = port_repo
        .count_ports(Some(asset.id), None, None, None)
        .await?;

    assert_eq!(count_after_delete, 0);

    Ok(())
}

#[sqlx::test]
async fn test_port_repository_filters(pool: PgPool) -> Result<()> {
    // Run migrations first
    setup_database(&pool).await?;

    let factory = RepositoryFactory::new(pool);
    let port_repo = factory.port_repository();

    // Create test organization and asset
    let org = create_test_organization(&factory, "Test Organization").await?;
    let asset1 = create_test_asset(&factory, org.id, AssetType::IPAddress, "192.168.1.1").await?;
    let asset2 = create_test_asset(&factory, org.id, AssetType::IPAddress, "192.168.1.2").await?;

    // Create multiple ports
    let ports = vec![
        Port::new(asset1.id, 80, Protocol::TCP, Some("http".to_string()), None),
        Port::new(
            asset1.id,
            443,
            Protocol::TCP,
            Some("https".to_string()),
            None,
        ),
        Port::new(asset1.id, 22, Protocol::TCP, Some("ssh".to_string()), None),
        Port::new(asset2.id, 80, Protocol::TCP, Some("http".to_string()), None),
        Port::new(asset2.id, 53, Protocol::UDP, Some("dns".to_string()), None),
    ];

    // Insert all ports
    for port in ports {
        port_repo.create_port(&port).await?;
    }

    // Test filter by asset_id
    let asset1_ports = port_repo
        .list_ports(Some(asset1.id), None, None, None, 10, 0)
        .await?;

    assert_eq!(asset1_ports.len(), 3);

    // Test filter by port_number
    let http_ports = port_repo
        .list_ports(None, Some(80), None, None, 10, 0)
        .await?;

    assert_eq!(http_ports.len(), 2);

    // Test filter by protocol
    let udp_ports = port_repo
        .list_ports(None, None, Some(Protocol::UDP), None, 10, 0)
        .await?;

    assert_eq!(udp_ports.len(), 1);
    assert_eq!(udp_ports[0].port_number, 53);

    // Test combined filters
    let asset1_tcp_ports = port_repo
        .list_ports(Some(asset1.id), None, Some(Protocol::TCP), None, 10, 0)
        .await?;

    assert_eq!(asset1_tcp_ports.len(), 3);

    Ok(())
}
