use clap::Parser;
use powernotify::config::Config;
use powernotify::power::{init_upower_proxy, PowerListener};
use smol::future;
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

static DEFAULT_CONFIG_PATH: LazyLock<String> =
    LazyLock::new(|| env::var("HOME").unwrap() + "/.config/powernotify/config.toml");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = DEFAULT_CONFIG_PATH.as_str()
    )]
    config: PathBuf,

    /// Triggers a test notification for a specific event from your config file without starting the daemon. This is useful for checking your configuration.
    /// Example Events: `ac`, `bat`, `20`, `5`
    #[arg(short, long, value_name = "EVENT")]
    test: Option<String>,
}

async fn test_event(config: &Config, event: &str) {
    let percent = event.parse::<i8>().unwrap_or(-1);

    let rule = match event {
        "ac" => config.rule_ac.as_ref(),
        "bat" => config.rule_bat.as_ref(),
        _ if (0..=100).contains(&percent) => config.percentage_rules.get(&percent),
        _ => {
            eprintln!("Incorrect test case");
            return;
        }
    };

    if let Some(rule) = rule {
        rule.execute().await;
    } else {
        eprintln!("No rule for given event");
    }
}

fn main() {
    let cli = Cli::parse();
    let canonicalized = match cli.config.canonicalize() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("{}: {}", err, cli.config.to_str().unwrap());
            return;
        }
    };

    let config = Config::get(&canonicalized);

    if let Some(event) = cli.test {
        smol::block_on(async {
            test_event(&config, event.as_str()).await;
        });
        return;
    }

    smol::block_on(async {
        let upower_proxy = init_upower_proxy().await.unwrap();

        let pl = PowerListener::new(&upower_proxy);

        let on_battery_fut = pl.listen_on_battery(async |is_on_battery| {
            let active_rule = if is_on_battery {
                &config.rule_bat
            } else {
                &config.rule_ac
            };

            if let Some(rule) = active_rule {
                eprintln!(
                    "Found rule on {}",
                    if is_on_battery { "battery" } else { "AC" }
                );
                rule.execute().await;
            };
        });
        let percentage_fut = pl.listen_percentage(async |percentage| {
            if let Some(rule) = config.percentage_rules.get(&percentage) {
                eprintln!("Found rule on {}%", percentage);
                rule.execute().await;
            }
        });

        let _ = future::zip(on_battery_fut, percentage_fut).await;
    });
}
