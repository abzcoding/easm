use async_trait::async_trait;
use rand;
use shared::types::{Severity, VulnerabilityStatus, ID};
use std::collections::HashMap;
use tokio;
use tracing::{debug, info};

use crate::{
    models::{Asset, Vulnerability},
    traits::{NotificationPeriod, NotificationService, NotificationSettings},
    Result,
};

/// Implementation of the NotificationService trait
pub struct NotificationServiceImpl {
    email_client: Option<EmailClient>,
    webhook_client: Option<WebhookClient>,
    settings_cache: HashMap<ID, NotificationSettings>,
}

/// Simple email client for notifications
struct EmailClient {
    // This would typically contain SMTP configuration or email service API client
}

impl EmailClient {
    fn new() -> Self {
        Self {}
    }

    async fn send_email(&self, to: &[String], subject: &str, body: &str) -> Result<bool> {
        // In a real implementation, this would connect to SMTP server or email API
        // For now, we'll just log it
        info!(
            "Would send email to {}, subject: {}, body: {}",
            to.join(", "),
            subject,
            body
        );
        Ok(true)
    }
}

/// Simple webhook client for notifications
struct WebhookClient {
    // This would typically contain HTTP client configuration
}

impl WebhookClient {
    fn new() -> Self {
        Self {}
    }

    async fn send_webhook(&self, url: &str, payload: &serde_json::Value) -> Result<bool> {
        // In a real implementation, this would send an HTTP POST
        // For now, we'll just log it
        info!(
            "Would send webhook to {}, payload: {}",
            url,
            serde_json::to_string_pretty(payload).unwrap_or_default()
        );
        Ok(true)
    }
}

impl NotificationServiceImpl {
    pub fn new() -> Self {
        Self {
            email_client: Some(EmailClient::new()),
            webhook_client: Some(WebhookClient::new()),
            settings_cache: HashMap::new(),
        }
    }

    /// Check if notification should be sent based on severity
    fn should_notify_severity(&self, severity: Severity, settings: &NotificationSettings) -> bool {
        severity >= settings.minimum_severity_for_notification
    }

    /// Create standard notification payload
    fn create_notification_payload(
        &self,
        event_type: &str,
        data: &serde_json::Value,
    ) -> serde_json::Value {
        let timestamp = chrono::Utc::now().to_rfc3339();

        serde_json::json!({
            "event_type": event_type,
            "timestamp": timestamp,
            "data": data
        })
    }

    /// Handle email sending with error handling and retries
    async fn send_email_with_retry(
        &self,
        client: &EmailClient,
        recipients: &[String],
        subject: &str,
        body: &str,
        max_retries: usize,
    ) -> Result<bool> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            match client.send_email(recipients, subject, body).await {
                Ok(success) => return Ok(success),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < max_retries {
                        // Exponential backoff with jitter
                        let backoff_ms = 100 * (1 << attempts) + rand::random::<u64>() % 100;
                        debug!(
                            "Email send failed, retrying in {}ms. Attempt {}/{}",
                            backoff_ms, attempts, max_retries
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        // All retries failed
        Err(last_error.unwrap_or_else(|| crate::Error::Internal("Unknown email error".to_string())))
    }

    /// Handle webhook sending with error handling and retries
    async fn send_webhook_with_retry(
        &self,
        client: &WebhookClient,
        url: &str,
        payload: &serde_json::Value,
        max_retries: usize,
    ) -> Result<bool> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            match client.send_webhook(url, payload).await {
                Ok(success) => return Ok(success),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < max_retries {
                        // Exponential backoff with jitter
                        let backoff_ms = 100 * (1 << attempts) + rand::random::<u64>() % 100;
                        debug!(
                            "Webhook send failed, retrying in {}ms. Attempt {}/{}",
                            backoff_ms, attempts, max_retries
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        // All retries failed
        Err(last_error
            .unwrap_or_else(|| crate::Error::Internal("Unknown webhook error".to_string())))
    }

    /// Send batch notifications for multiple vulnerabilities
    async fn send_batch_notification(
        &self,
        vulnerabilities: &[Vulnerability],
        event_type: &str,
        organization_id: ID,
    ) -> Result<bool> {
        if vulnerabilities.is_empty() {
            return Ok(true);
        }

        debug!(
            "Preparing batch notification for {} vulnerabilities",
            vulnerabilities.len()
        );

        // Get notification settings
        let settings = self.get_notification_settings(organization_id).await?;

        // Check notification settings
        let should_notify = match event_type {
            "new_vulnerability" => settings.notify_on_new_vulnerability,
            "vulnerability_status_change" => settings.notify_on_status_change,
            _ => true, // Other event types are always notified
        };

        if !should_notify {
            debug!("Batch notification skipped based on settings");
            return Ok(false);
        }

        // Filter vulnerabilities based on severity
        let filtered_vulns: Vec<&Vulnerability> = vulnerabilities
            .iter()
            .filter(|v| self.should_notify_severity(v.severity, &settings))
            .collect();

        if filtered_vulns.is_empty() {
            debug!("No vulnerabilities meet severity threshold for notification");
            return Ok(false);
        }

        // Count vulnerabilities by severity
        let mut severity_counts: HashMap<Severity, usize> = HashMap::new();
        for vuln in &filtered_vulns {
            *severity_counts.entry(vuln.severity).or_insert(0) += 1;
        }

        // Build email content
        let subject = format!(
            "[EASM] Batch notification: {} new findings",
            filtered_vulns.len()
        );

        let mut body = "The following vulnerabilities have been detected:\n\n".to_string();

        // Add severity summary
        body.push_str("Summary by severity:\n");
        for (severity, count) in severity_counts.iter() {
            body.push_str(&format!("- {:?}: {}\n", severity, count));
        }
        body.push_str("\n\nDetails:\n");

        // Add details for each vulnerability
        for (i, vuln) in filtered_vulns.iter().enumerate() {
            body.push_str(&format!(
                "\n{}. {} (Severity: {:?})\n   Asset ID: {}\n   Description: {}\n",
                i + 1,
                vuln.title,
                vuln.severity,
                vuln.asset_id,
                vuln.description
                    .as_deref()
                    .unwrap_or("No description available")
            ));
        }

        // Build webhook payload
        let payload = self.create_notification_payload(
            &format!("batch_{}", event_type),
            &serde_json::json!({
                "count": filtered_vulns.len(),
                "severity_summary": severity_counts,
                "vulnerabilities": filtered_vulns,
            }),
        );

        // Send notifications based on settings
        let mut success = true;
        let max_retries = 3;

        if settings.email_notifications && !settings.email_recipients.is_empty() {
            if let Some(client) = &self.email_client {
                match self
                    .send_email_with_retry(
                        client,
                        &settings.email_recipients,
                        &subject,
                        &body,
                        max_retries,
                    )
                    .await
                {
                    Ok(_) => debug!("Batch email notification sent successfully"),
                    Err(e) => {
                        debug!("Failed to send batch email notification: {:?}", e);
                        success = false;
                    }
                }
            }
        }

        if settings.webhook_notifications {
            if let Some(url) = &settings.webhook_url {
                if let Some(client) = &self.webhook_client {
                    match self
                        .send_webhook_with_retry(client, url, &payload, max_retries)
                        .await
                    {
                        Ok(_) => debug!("Batch webhook notification sent successfully"),
                        Err(e) => {
                            debug!("Failed to send batch webhook notification: {:?}", e);
                            success = false;
                        }
                    }
                }
            }
        }

        Ok(success)
    }
}

impl Default for NotificationServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn notify_new_vulnerability(&self, vulnerability: &Vulnerability) -> Result<bool> {
        info!(
            "Preparing notification for new vulnerability: {}",
            vulnerability.id
        );

        // Get notification settings
        let settings = self
            .get_notification_settings(vulnerability.asset_id)
            .await?;

        // Check if we should notify based on settings
        if !settings.notify_on_new_vulnerability
            || !self.should_notify_severity(vulnerability.severity, &settings)
        {
            debug!("Notification skipped based on settings");
            return Ok(false);
        }

        // Build email content
        let subject = format!(
            "[EASM] New {} vulnerability detected: {}",
            format!("{:?}", vulnerability.severity).to_uppercase(),
            vulnerability.title
        );

        let body = format!(
            "A new vulnerability has been detected:\n\nTitle: {}\nSeverity: {:?}\nAsset ID: {}\n\nDescription: {}\n\nRemediation: {}",
            vulnerability.title,
            vulnerability.severity,
            vulnerability.asset_id,
            vulnerability.description.as_deref().unwrap_or("No description available"),
            vulnerability.remediation.as_deref().unwrap_or("No remediation information available")
        );

        // Build webhook payload
        let payload = self.create_notification_payload(
            "new_vulnerability",
            &serde_json::to_value(vulnerability).unwrap_or(serde_json::Value::Null),
        );

        // Send notifications based on settings
        let mut success = true;
        let max_retries = 3;

        if settings.email_notifications && !settings.email_recipients.is_empty() {
            if let Some(client) = &self.email_client {
                match self
                    .send_email_with_retry(
                        client,
                        &settings.email_recipients,
                        &subject,
                        &body,
                        max_retries,
                    )
                    .await
                {
                    Ok(_) => debug!("Email notification sent successfully"),
                    Err(e) => {
                        debug!("Failed to send email notification: {:?}", e);
                        success = false;
                    }
                }
            }
        }

        if settings.webhook_notifications {
            if let Some(url) = &settings.webhook_url {
                if let Some(client) = &self.webhook_client {
                    match self
                        .send_webhook_with_retry(client, url, &payload, max_retries)
                        .await
                    {
                        Ok(_) => debug!("Webhook notification sent successfully"),
                        Err(e) => {
                            debug!("Failed to send webhook notification: {:?}", e);
                            success = false;
                        }
                    }
                }
            }
        }

        Ok(success)
    }

    // New method to handle batch notifications for multiple new vulnerabilities
    async fn notify_new_vulnerabilities_batch(
        &self,
        vulnerabilities: &[Vulnerability],
    ) -> Result<bool> {
        if vulnerabilities.is_empty() {
            return Ok(true);
        }

        // Group vulnerabilities by organization for more targeted notifications
        let mut org_vulns: HashMap<ID, Vec<Vulnerability>> = HashMap::new();
        for vuln in vulnerabilities {
            org_vulns
                .entry(vuln.asset_id)
                .or_default()
                .push(vuln.clone());
        }

        let mut overall_success = true;

        // Send batch notifications for each organization
        for (org_id, vulns) in org_vulns {
            let success = self
                .send_batch_notification(&vulns, "new_vulnerability", org_id)
                .await?;
            if !success {
                overall_success = false;
            }
        }

        Ok(overall_success)
    }

    async fn notify_vulnerability_status_change(
        &self,
        vulnerability: &Vulnerability,
        old_status: VulnerabilityStatus,
    ) -> Result<bool> {
        info!(
            "Preparing notification for vulnerability status change: {} ({:?} -> {:?})",
            vulnerability.id, old_status, vulnerability.status
        );

        // Get notification settings
        let settings = self
            .get_notification_settings(vulnerability.asset_id)
            .await?;

        // Check if we should notify based on settings
        if !settings.notify_on_status_change
            || !self.should_notify_severity(vulnerability.severity, &settings)
        {
            debug!("Notification skipped based on settings");
            return Ok(false);
        }

        // Build email content
        let subject = format!(
            "[EASM] Vulnerability status changed: {}",
            vulnerability.title
        );

        let body = format!(
            "A vulnerability status has changed:\n\nTitle: {}\nSeverity: {:?}\nStatus: {:?} -> {:?}\nAsset ID: {}\n\nDescription: {}",
            vulnerability.title,
            vulnerability.severity,
            old_status,
            vulnerability.status,
            vulnerability.asset_id,
            vulnerability.description.as_deref().unwrap_or("No description available")
        );

        // Build webhook payload
        let mut vuln_value = serde_json::to_value(vulnerability).unwrap_or(serde_json::Value::Null);

        if let serde_json::Value::Object(ref mut map) = vuln_value {
            map.insert(
                "old_status".to_string(),
                serde_json::Value::String(format!("{:?}", old_status)),
            );
        }

        let payload = self.create_notification_payload("vulnerability_status_change", &vuln_value);

        // Send notifications based on settings
        let mut success = true;

        if settings.email_notifications && !settings.email_recipients.is_empty() {
            if let Some(client) = &self.email_client {
                if let Err(e) = client
                    .send_email(&settings.email_recipients, &subject, &body)
                    .await
                {
                    debug!("Failed to send email notification: {:?}", e);
                    success = false;
                }
            }
        }

        if settings.webhook_notifications {
            if let Some(url) = &settings.webhook_url {
                if let Some(client) = &self.webhook_client {
                    if let Err(e) = client.send_webhook(url, &payload).await {
                        debug!("Failed to send webhook notification: {:?}", e);
                        success = false;
                    }
                }
            }
        }

        Ok(success)
    }

    async fn notify_new_critical_asset(&self, asset: &Asset) -> Result<bool> {
        info!(
            "Preparing notification for new critical asset: {} ({})",
            asset.id, asset.value
        );

        // Get notification settings
        let settings = self
            .get_notification_settings(asset.organization_id)
            .await?;

        // Check if we should notify based on settings
        if !settings.notify_on_new_critical_asset {
            debug!("Notification skipped based on settings");
            return Ok(false);
        }

        // Build email content
        let subject = format!("[EASM] New critical asset discovered: {}", asset.value);

        let body = format!(
            "A new critical asset has been discovered:\n\nAsset: {}\nType: {:?}\nID: {}\nOrganization ID: {}",
            asset.value,
            asset.asset_type,
            asset.id,
            asset.organization_id
        );

        // Build webhook payload
        let payload = self.create_notification_payload(
            "new_critical_asset",
            &serde_json::to_value(asset).unwrap_or(serde_json::Value::Null),
        );

        // Send notifications based on settings
        let mut success = true;

        if settings.email_notifications && !settings.email_recipients.is_empty() {
            if let Some(client) = &self.email_client {
                if let Err(e) = client
                    .send_email(&settings.email_recipients, &subject, &body)
                    .await
                {
                    debug!("Failed to send email notification: {:?}", e);
                    success = false;
                }
            }
        }

        if settings.webhook_notifications {
            if let Some(url) = &settings.webhook_url {
                if let Some(client) = &self.webhook_client {
                    if let Err(e) = client.send_webhook(url, &payload).await {
                        debug!("Failed to send webhook notification: {:?}", e);
                        success = false;
                    }
                }
            }
        }

        Ok(success)
    }

    async fn send_summary_report(
        &self,
        organization_id: ID,
        period: NotificationPeriod,
    ) -> Result<bool> {
        info!(
            "Preparing summary report for organization: {}, period: {:?}",
            organization_id, period
        );

        // Get notification settings
        let settings = self.get_notification_settings(organization_id).await?;

        if !settings.email_notifications && !settings.webhook_notifications {
            debug!("No notification methods enabled, skipping summary report");
            return Ok(false);
        }

        // In a real implementation, we would generate a report based on recent data
        // For now, we'll create a placeholder report

        let period_str = match period {
            NotificationPeriod::Daily => "Daily",
            NotificationPeriod::Weekly => "Weekly",
            NotificationPeriod::Monthly => "Monthly",
        };

        let subject = format!("[EASM] {} Security Summary Report", period_str);

        let body = format!(
            "{} Security Summary Report for Organization: {}\n\n",
            period_str, organization_id
        );

        let payload = self.create_notification_payload(
            &format!("{}_summary_report", period_str.to_lowercase()),
            &serde_json::json!({
                "organization_id": organization_id.to_string(),
                "period": period_str,
                "generated_at": chrono::Utc::now().to_rfc3339(),
                // In a real implementation, this would include actual statistics
                "stats": {
                    "new_vulnerabilities": 0,
                    "resolved_vulnerabilities": 0,
                    "critical_vulnerabilities": 0,
                    "high_vulnerabilities": 0,
                    "new_assets": 0
                }
            }),
        );

        // Send notifications based on settings
        let mut success = true;

        if settings.email_notifications && !settings.email_recipients.is_empty() {
            if let Some(client) = &self.email_client {
                if let Err(e) = client
                    .send_email(&settings.email_recipients, &subject, &body)
                    .await
                {
                    debug!("Failed to send email notification: {:?}", e);
                    success = false;
                }
            }
        }

        if settings.webhook_notifications {
            if let Some(url) = &settings.webhook_url {
                if let Some(client) = &self.webhook_client {
                    if let Err(e) = client.send_webhook(url, &payload).await {
                        debug!("Failed to send webhook notification: {:?}", e);
                        success = false;
                    }
                }
            }
        }

        Ok(success)
    }

    async fn get_notification_settings(&self, organization_id: ID) -> Result<NotificationSettings> {
        // In a real implementation, this would query the database
        // For now, we'll return default settings
        let settings = NotificationSettings {
            organization_id,
            email_notifications: true,
            email_recipients: vec!["security@example.com".to_string()],
            webhook_notifications: false,
            webhook_url: None,
            notification_period: NotificationPeriod::Daily,
            notify_on_new_vulnerability: true,
            notify_on_status_change: true,
            notify_on_new_critical_asset: true,
            minimum_severity_for_notification: Severity::Medium,
            additional_settings: None,
        };

        Ok(settings)
    }

    async fn update_notification_settings(
        &self,
        organization_id: ID,
        settings: &NotificationSettings,
    ) -> Result<NotificationSettings> {
        // In a real implementation, this would update the database
        // For now, we'll just return the same settings
        info!(
            "Would update notification settings for organization: {}",
            organization_id
        );

        Ok(settings.clone())
    }
}
