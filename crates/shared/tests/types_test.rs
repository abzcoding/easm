#[cfg(test)]
mod tests {
    use serde_json::{from_value, json, to_value};
    use shared::types::{
        AssetStatus, AssetType, JobStatus, JobType, PortStatus, Protocol, Severity, UserRole,
        VulnerabilityStatus,
    };

    #[test]
    fn test_asset_status_serialization() {
        let status = AssetStatus::Active;
        let json = to_value(status).unwrap();
        assert_eq!(json, json!("ACTIVE"));

        let deserialized: AssetStatus = from_value(json).unwrap();
        assert_eq!(deserialized, AssetStatus::Active);
    }

    #[test]
    fn test_asset_type_serialization() {
        let asset_type = AssetType::Domain;
        let json = to_value(asset_type).unwrap();
        assert_eq!(json, json!("DOMAIN"));

        let deserialized: AssetType = from_value(json).unwrap();
        assert_eq!(deserialized, AssetType::Domain);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Critical;
        let json = to_value(severity).unwrap();
        assert_eq!(json, json!("CRITICAL"));

        let deserialized: Severity = from_value(json).unwrap();
        assert_eq!(deserialized, Severity::Critical);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn test_vulnerability_status_serialization() {
        let status = VulnerabilityStatus::Open;
        let json = to_value(status).unwrap();
        assert_eq!(json, json!("OPEN"));

        let deserialized: VulnerabilityStatus = from_value(json).unwrap();
        assert_eq!(deserialized, VulnerabilityStatus::Open);
    }

    #[test]
    fn test_protocol_serialization() {
        let protocol = Protocol::TCP;
        let json = to_value(protocol).unwrap();
        assert_eq!(json, json!("TCP"));

        let deserialized: Protocol = from_value(json).unwrap();
        assert_eq!(deserialized, Protocol::TCP);
    }

    #[test]
    fn test_port_status_serialization() {
        let status = PortStatus::Open;
        let json = to_value(status).unwrap();
        assert_eq!(json, json!("OPEN"));

        let deserialized: PortStatus = from_value(json).unwrap();
        assert_eq!(deserialized, PortStatus::Open);
    }

    #[test]
    fn test_job_type_serialization() {
        let job_type = JobType::DnsEnum;
        let json = to_value(job_type).unwrap();
        assert_eq!(json, json!("DNSENUM"));

        let deserialized: JobType = from_value(json).unwrap();
        assert_eq!(deserialized, JobType::DnsEnum);
    }

    #[test]
    fn test_job_status_serialization() {
        let status = JobStatus::Running;
        let json = to_value(status).unwrap();
        assert_eq!(json, json!("RUNNING"));

        let deserialized: JobStatus = from_value(json).unwrap();
        assert_eq!(deserialized, JobStatus::Running);
    }

    #[test]
    fn test_user_role_serialization() {
        let role = UserRole::Admin;
        let json = to_value(role).unwrap();
        assert_eq!(json, json!("ADMIN"));

        let deserialized: UserRole = from_value(json).unwrap();
        assert_eq!(deserialized, UserRole::Admin);
    }
}
