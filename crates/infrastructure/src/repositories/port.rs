use crate::utils::{from_offset_datetime, to_offset_datetime};
use async_trait::async_trait;
use backend::{models::Port, traits::PortRepository, Result};
use shared::types::{PortStatus, Protocol, ID};
use sqlx::PgPool;

/// PostgreSQL implementation of the Port Repository
pub struct PgPortRepository {
    pool: PgPool,
}

impl PgPortRepository {
    /// Create a new PgPortRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PortRepository for PgPortRepository {
    async fn create_port(&self, port: &Port) -> Result<Port> {
        // Convert DateTime types for database operation
        let first_seen = to_offset_datetime(port.first_seen);
        let last_seen = to_offset_datetime(port.last_seen);
        let created_at = to_offset_datetime(port.created_at);
        let updated_at = to_offset_datetime(port.updated_at);

        let record = sqlx::query!(
            r#"
            INSERT INTO ports (id, asset_id, port_number, protocol, service_name, banner, status, first_seen, last_seen, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
            "#,
            port.id,
            port.asset_id,
            port.port_number,
            port.protocol as Protocol,
            port.service_name,
            port.banner,
            port.status as PortStatus,
            first_seen,
            last_seen,
            created_at,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(Port {
            id: record.id,
            asset_id: record.asset_id,
            port_number: record.port_number,
            protocol: record.protocol,
            service_name: record.service_name,
            banner: record.banner,
            status: record.status.expect("Port status should not be null"),
            first_seen: from_offset_datetime(Some(record.first_seen)),
            last_seen: from_offset_datetime(Some(record.last_seen)),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn get_port(&self, id: ID) -> Result<Port> {
        let record = sqlx::query!(
            r#"
            SELECT id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
            FROM ports
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(Port {
            id: record.id,
            asset_id: record.asset_id,
            port_number: record.port_number,
            protocol: record.protocol,
            service_name: record.service_name,
            banner: record.banner,
            status: record.status.expect("Port status should not be null"),
            first_seen: from_offset_datetime(Some(record.first_seen)),
            last_seen: from_offset_datetime(Some(record.last_seen)),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn update_port(&self, port: &Port) -> Result<Port> {
        // Convert DateTime types for database operation
        let first_seen = to_offset_datetime(port.first_seen);
        let last_seen = to_offset_datetime(port.last_seen);
        let updated_at = to_offset_datetime(port.updated_at);

        let record = sqlx::query!(
            r#"
            UPDATE ports
            SET asset_id = $2, port_number = $3, protocol = $4, service_name = $5, banner = $6, status = $7, first_seen = $8, last_seen = $9, updated_at = $10
            WHERE id = $1
            RETURNING id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
            "#,
            port.id,
            port.asset_id,
            port.port_number,
            port.protocol as Protocol,
            port.service_name,
            port.banner,
            port.status as PortStatus,
            first_seen,
            last_seen,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(Port {
            id: record.id,
            asset_id: record.asset_id,
            port_number: record.port_number,
            protocol: record.protocol,
            service_name: record.service_name,
            banner: record.banner,
            status: record.status.expect("Port status should not be null"),
            first_seen: from_offset_datetime(Some(record.first_seen)),
            last_seen: from_offset_datetime(Some(record.last_seen)),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn delete_port(&self, id: ID) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM ports
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_ports(
        &self,
        asset_id: Option<ID>,
        port_number: Option<i32>,
        protocol: Option<Protocol>,
        status: Option<PortStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Port>> {
        // Filter by asset_id only
        if let Some(asset_id) = asset_id {
            if port_number.is_none() && protocol.is_none() && status.is_none() {
                let records = sqlx::query!(
                    r#"
                    SELECT id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
                    FROM ports
                    WHERE asset_id = $1
                    ORDER BY port_number
                    LIMIT $2 OFFSET $3
                    "#,
                    asset_id,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                return Ok(records
                    .into_iter()
                    .map(|record| Port {
                        id: record.id,
                        asset_id: record.asset_id,
                        port_number: record.port_number,
                        protocol: record.protocol,
                        service_name: record.service_name,
                        banner: record.banner,
                        status: record.status.expect("Port status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect());
            }

            // asset_id + protocol
            if let Some(proto) = protocol {
                let records = sqlx::query!(
                    r#"
                    SELECT id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
                    FROM ports
                    WHERE asset_id = $1 AND protocol = $2
                    ORDER BY port_number
                    LIMIT $3 OFFSET $4
                    "#,
                    asset_id,
                    proto as Protocol,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                return Ok(records
                    .into_iter()
                    .map(|record| Port {
                        id: record.id,
                        asset_id: record.asset_id,
                        port_number: record.port_number,
                        protocol: record.protocol,
                        service_name: record.service_name,
                        banner: record.banner,
                        status: record.status.expect("Port status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect());
            }
        }

        // Filter by port_number only
        if let Some(port_num) = port_number {
            if asset_id.is_none() && protocol.is_none() && status.is_none() {
                let records = sqlx::query!(
                    r#"
                    SELECT id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
                    FROM ports
                    WHERE port_number = $1
                    ORDER BY port_number
                    LIMIT $2 OFFSET $3
                    "#,
                    port_num,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                return Ok(records
                    .into_iter()
                    .map(|record| Port {
                        id: record.id,
                        asset_id: record.asset_id,
                        port_number: record.port_number,
                        protocol: record.protocol,
                        service_name: record.service_name,
                        banner: record.banner,
                        status: record.status.expect("Port status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect());
            }
        }

        // Filter by protocol only
        if let Some(proto) = protocol {
            if asset_id.is_none() && port_number.is_none() && status.is_none() {
                let records = sqlx::query!(
                    r#"
                    SELECT id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
                    FROM ports
                    WHERE protocol = $1
                    ORDER BY port_number
                    LIMIT $2 OFFSET $3
                    "#,
                    proto as Protocol,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                return Ok(records
                    .into_iter()
                    .map(|record| Port {
                        id: record.id,
                        asset_id: record.asset_id,
                        port_number: record.port_number,
                        protocol: record.protocol,
                        service_name: record.service_name,
                        banner: record.banner,
                        status: record.status.expect("Port status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect());
            }
        }

        // No filters - return all
        let records = sqlx::query!(
            r#"
            SELECT id, asset_id, port_number, protocol as "protocol: Protocol", service_name, banner, status as "status: PortStatus", first_seen, last_seen, created_at, updated_at
            FROM ports
            ORDER BY port_number
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|record| Port {
                id: record.id,
                asset_id: record.asset_id,
                port_number: record.port_number,
                protocol: record.protocol,
                service_name: record.service_name,
                banner: record.banner,
                status: record.status.expect("Port status should not be null"),
                first_seen: from_offset_datetime(Some(record.first_seen)),
                last_seen: from_offset_datetime(Some(record.last_seen)),
                created_at: from_offset_datetime(Some(record.created_at)),
                updated_at: from_offset_datetime(Some(record.updated_at)),
            })
            .collect())
    }

    async fn count_ports(
        &self,
        asset_id: Option<ID>,
        port_number: Option<i32>,
        protocol: Option<Protocol>,
        status: Option<PortStatus>,
    ) -> Result<usize> {
        // Filter by asset_id only
        if let Some(asset_id) = asset_id {
            if port_number.is_none() && protocol.is_none() && status.is_none() {
                let count = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ports
                    WHERE asset_id = $1
                    "#,
                    asset_id
                )
                .fetch_one(&self.pool)
                .await?;

                return Ok(count as usize);
            }

            // asset_id + protocol
            if let Some(proto) = protocol {
                let count = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ports
                    WHERE asset_id = $1 AND protocol = $2
                    "#,
                    asset_id,
                    proto as Protocol
                )
                .fetch_one(&self.pool)
                .await?;

                return Ok(count as usize);
            }
        }

        // Filter by port_number only
        if let Some(port_num) = port_number {
            if asset_id.is_none() && protocol.is_none() && status.is_none() {
                let count = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ports
                    WHERE port_number = $1
                    "#,
                    port_num
                )
                .fetch_one(&self.pool)
                .await?;

                return Ok(count as usize);
            }
        }

        // Filter by protocol only
        if let Some(proto) = protocol {
            if asset_id.is_none() && port_number.is_none() && status.is_none() {
                let count = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ports
                    WHERE protocol = $1
                    "#,
                    proto as Protocol
                )
                .fetch_one(&self.pool)
                .await?;

                return Ok(count as usize);
            }
        }

        // No filters - count all ports
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM ports
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count as usize)
    }
}
