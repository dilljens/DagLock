//! Email notification service for DagLock.
//! Sends transactional emails via SMTP when escrow events occur.
//! Opt-in per user, rate-limited to 10 emails/address/day.

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared email state.
pub struct EmailService {
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_user: Option<String>,
    smtp_pass: Option<String>,
    from_addr: Option<String>,
    base_url: String,
    rate_limiter: Arc<Mutex<Vec<(String, i64)>>>, // (address, timestamp)
}

impl EmailService {
    pub fn new(
        smtp_host: Option<String>,
        smtp_port: Option<u16>,
        smtp_user: Option<String>,
        smtp_pass: Option<String>,
        from_addr: Option<String>,
        base_url: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            from_addr,
            base_url,
            rate_limiter: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.smtp_host.is_some() && self.from_addr.is_some()
    }

    /// Check rate limit: max 10 emails per address per day.
    pub async fn check_rate_limit(&self, address: &str) -> bool {
        let now = chrono::Utc::now().timestamp();
        let mut limiter = self.rate_limiter.lock().await;
        // Remove entries older than 24h
        limiter.retain(|(addr, ts)| addr == address && *ts > now - 86400);
        limiter.push((address.to_string(), now));
        limiter.len() <= 10
    }

    /// Send an email about an escrow event.
    pub async fn send_notification(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<Response, String> {
        let host = self.smtp_host.as_ref().ok_or("SMTP not configured")?;
        let from = self.from_addr.as_ref().ok_or("From address not configured")?;
        let port = self.smtp_port.unwrap_or(587);

        let email = Message::builder()
            .from(from.parse().map_err(|e| format!("Invalid from address: {e}"))?)
            .to(to_email.parse().map_err(|e| format!("Invalid to address: {e}"))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {e}"))?;

        let creds = match (&self.smtp_user, &self.smtp_pass) {
            (Some(user), Some(pass)) => Some(Credentials::new(user.clone(), pass.clone())),
            _ => None,
        };

        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .map_err(|e| format!("SMTP relay error: {e}"))?
            .port(port)
            .credentials(creds.unwrap_or_else(|| Credentials::new("".to_string(), "".to_string())))
            .build();

        transport.send(email).await.map_err(|e| format!("SMTP send error: {e}"))
    }

    /// Build and send an escrow event notification.
    pub async fn notify_escrow_event(
        &self,
        email: &str,
        address: &str,
        event_type: &str,
        escrow_id: &str,
        amount_sompi: i64,
        status: &str,
    ) -> Result<(), String> {
        if !self.check_rate_limit(address).await {
            return Err("Rate limit exceeded (max 10 emails/day)".to_string());
        }

        let amount_kas = amount_sompi as f64 / 100_000_000.0;
        let escrow_url = format!("{}/escrows?id={}", self.base_url, escrow_id);

        let subject = format!("[DagLock] Escrow {} — {}", escrow_id, event_type);
        let body = format!(
            "DagLock Escrow Notification\n\
             ===========================\n\n\
             Event: {}\n\
             Escrow: {}\n\
             Amount: {:.2} KAS\n\
             Status: {}\n\n\
             View details:\n\
             {}\n\n\
             ---\n\
             DagLock — Trustless Escrow on Kaspa\n\
             https://daglock.com\n\
             This notification was sent because you subscribed to escrow alerts.\
             To unsubscribe, visit: {}/settings",
            event_type, escrow_id, amount_kas, status, escrow_url, self.base_url
        );

        self.send_notification(email, &subject, &body).await?;
        Ok(())
    }

    /// Send email verification code.
    pub async fn send_verification(
        &self,
        email: &str,
        _address: &str,
        code: &str,
    ) -> Result<(), String> {
        let subject = "Verify your email for DagLock notifications";
        let body = format!(
            "Welcome to DagLock email notifications!\n\n\
             Your verification code: {}\n\n\
             Enter this code on the DagLock settings page to verify your email.\n\n\
             If you didn't request this, ignore this email.\n\n\
             ---\n\
             DagLock — Trustless Escrow on Kaspa\n\
             https://daglock.com",
            code
        );

        self.send_notification(email, subject, &body).await?;
        Ok(())
    }
}
