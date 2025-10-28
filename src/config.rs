use crate::rules::{PercentageRules, Rule};
use indexmap::IndexMap;
use notify_rust::{Notification, Urgency};
use serde::Deserialize;
use std::fs;
use std::time::Duration;
use std::{collections::HashMap, path::Path};

pub struct Config {
    pub rule_ac: Option<Rule>,
    pub rule_bat: Option<Rule>,
    pub percentage_rules: PercentageRules,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ConfigUrgency {
    Low,
    Normal,
    Critical,
}

#[derive(Deserialize, Debug)]
struct ConfigRule {
    cmd: Option<String>,
    summary: Option<String>,
    body: Option<String>,
    icon: Option<String>,
    appname: Option<String>,
    urgency: Option<ConfigUrgency>,
    actions: Option<IndexMap<String, String>>,
    timeout: Option<u64>,
    //sound_name: Option<String>,
    //hint: Option<Hint>,
    //action: Option<String>,
    //action_name: Option<String>,
    //#[serde(flatten)]
    //extra: HashMap<String, Value>,
}

impl Config {
    pub fn get(path: &Path) -> Self {
        let content = fs::read_to_string(path).unwrap();

        let map =
            toml::from_str::<HashMap<String, ConfigRule>>(content.to_owned().as_str()).unwrap();

        let mut rule_ac: Option<Rule> = None;
        let mut rule_bat: Option<Rule> = None;
        let mut percentage_rules: PercentageRules = HashMap::new();

        let exec_in_dir = path.parent().unwrap_or(Path::new("/")).to_path_buf();

        for (k, v) in map {
            //if !v.extra.is_empty() {
            //    let collected = v.extra.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
            //    eprintln!("Unknown fields in config: {}", collected.join(", "));
            //}

            let build_rule = || {
                Rule::new(
                    build_notification(&v),
                    v.cmd,
                    v.actions,
                    exec_in_dir.to_owned(),
                )
            };

            let percent = k.parse::<i8>().unwrap_or(-1);

            match k.as_str() {
                "ac" => {
                    rule_ac = Some(build_rule());
                }
                "bat" => {
                    rule_bat = Some(build_rule());
                }
                _ if (0..=100).contains(&percent) => {
                    percentage_rules.insert(percent, build_rule());
                }
                _ => (),
            }
        }

        Self {
            rule_ac,
            rule_bat,
            percentage_rules,
        }
    }
}

fn build_notification(rule: &ConfigRule) -> Option<Notification> {
    let mut notification = Notification::new();
    let mut edited = false;

    if let Some(summary) = &rule.summary {
        notification.summary(summary);
        edited = true;
    }

    if let Some(body) = &rule.body {
        notification.body(body);
        edited = true;
    }

    if let Some(icon) = &rule.icon {
        notification.icon(icon);
        edited = true;
    }

    if let Some(appname) = &rule.appname {
        notification.appname(appname);
        edited = true;
    }

    if let Some(urgency) = &rule.urgency {
        let urgency = match urgency {
            ConfigUrgency::Low => Urgency::Low,
            ConfigUrgency::Normal => Urgency::Normal,
            ConfigUrgency::Critical => Urgency::Critical,
        };
        notification.urgency(urgency);
        edited = true;
    }

    if let Some(actions) = &rule.actions {
        for (label, _) in actions {
            notification.action(label, label);
        }
        edited = true;
    }

    if let Some(timeout) = &rule.timeout {
        notification.timeout(Duration::from_secs(*timeout));
    }

    if edited {
        Some(notification.finalize())
    } else {
        None
    }
}
