//! src/email_client.rs
use crate::routes::subscriptions::SubscriberEmail;
use reqwest::Client;
use serde::Serialize;

#[derive(Clone)]
pub struct EmailClient {
    pub sender: SubscriberEmail,
    pub url: String,
    pub token: String,
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: String,
        html_content: String,
        text_content: String,
    ) -> Result<(), String> {
        let client = Client::new();

        let payload = SendEmailRequest {
            from: self.sender.clone(),
            to: recipient.clone(),
            subject: subject.clone(),
            html_body: html_content.clone(),
            text_body: text_content.clone(),
        };
        let res = client
            .post(self.url.clone()) // TODO: load from config
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("X-Postmark-Server-Token", self.token.clone()) // TODO: load
            // from config
            .json(&payload)
            .send()
            .await;

        Ok(())
    }
}

#[derive(Serialize)]
struct SendEmailRequest {
    from: SubscriberEmail,
    to: SubscriberEmail,
    subject: String,
    html_body: String,
    text_body: String,
}
