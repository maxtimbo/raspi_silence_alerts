use anyhow::{Result, Context};
use mail_builder::headers::address::Address;
use mail_builder::MessageBuilder;
use mail_send::{Credentials, SmtpClientBuilder};

use crate::conf::MailConfig;

impl MailConfig {
    pub async fn send_mail(&self, subject: &str, body: &str) -> Result<()> {
        let msg = MessageBuilder::new()
            .from(Address::new_address(String::new().into(), &self.fromname))
            .to(&*self.recipient)
            .subject(subject)
            .text_body(body);

        let result = SmtpClientBuilder::new(&self.smtp_server, self.port)
            .implicit_tls(self.tls)
            .credentials(Credentials::new(&self.username, &self.password))
            .connect()
            .await
            .context("Failed to connect to SMTP server")?
            .send(msg)
            .await;
        match result {
            Ok(_) => println!("Email sent to {}", self.recipient),
            Err(e) => eprintln!("error {:?}", e)
        }
        Ok(())
    }
}


