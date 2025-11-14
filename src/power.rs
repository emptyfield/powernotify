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

    pub async fn listen_on_battery<F, Fut>(&self, handler: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(bool) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut on_battery_stream = self.upower.receive_on_battery_changed().await;

        let _: () = while let Some(event) = on_battery_stream.next().await {
            handler(event.get().await?).await;
        };
        Ok(())
    }

    pub async fn listen_percentage<F, Fut>(&self, handler: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(i8) -> Fut,
        Fut: Future<Output = ()>,
    {
        let display_device = self.upower.get_display_device().await?;
        let mut percentage_stream = display_device.receive_percentage_changed().await;

        let _: () = while let Some(event) = percentage_stream.next().await {
            let percentage = event.get().await? as i8;

            handler(percentage).await;
        };
        Ok(())
    }
}
