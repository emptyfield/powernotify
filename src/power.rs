use crate::rules::{PercentageRules, Rule};
use smol::stream::StreamExt;
use std::error::Error;
use upower_dbus::UPowerProxy;
use zbus::Connection;

pub async fn init_upower_proxy() -> zbus::Result<UPowerProxy<'static>> {
    let conn = Connection::system().await?;
    UPowerProxy::new(&conn).await
}

pub struct PowerListener<'a> {
    upower: &'a UPowerProxy<'a>,
}

impl<'a> PowerListener<'a> {
    pub fn new(upower_proxy: &'a UPowerProxy) -> Self {
        Self {
            upower: upower_proxy,
        }
    }

    pub async fn listen_on_battery(
        &self,
        rule_on_ac: Option<Rule>,
        rule_on_bat: Option<Rule>,
    ) -> Result<(), Box<dyn Error>> {
        let mut on_battery_stream = self.upower.receive_on_battery_changed().await;

        let _: () = while let Some(event) = on_battery_stream.next().await {
            let is_on_battery = event.get().await?;

            let active_rule = if is_on_battery {
                &rule_on_bat
            } else {
                &rule_on_ac
            };

            if let Some(rule) = active_rule {
                eprintln!(
                    "Found rule on {}",
                    if is_on_battery { "battery" } else { "AC" }
                );
                rule.execute().await;
            }
        };
        Ok(())
    }

    pub async fn listen_percentage(&self, rules: &PercentageRules) -> Result<(), Box<dyn Error>> {
        let display_device = self.upower.get_display_device().await?;
        let mut percentage_stream = display_device.receive_percentage_changed().await;

        let _: () = while let Some(event) = percentage_stream.next().await {
            let percentage = event.get().await? as i8;

            if let Some(rule) = rules.get(&percentage) {
                eprintln!("Found rule on {}%", percentage);
                rule.execute().await;
            }
        };
        Ok(())
    }
}
