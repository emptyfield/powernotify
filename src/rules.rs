use indexmap::IndexMap;
use notify_rust::Notification;
use smol::{future::zip, process::Command, unblock};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static SHELL_PATH: LazyLock<String> = LazyLock::new(|| env::var("SHELL").unwrap());

async fn run_command(exec_in_dir: &Path, cmd: &str) {
    let output = Command::new(&*SHELL_PATH)
        .current_dir(exec_in_dir)
        .arg("-c")
        .arg(cmd)
        .output()
        .await;

    if let Err(err) = output {
        eprintln!("Error while executing {}:\n{}", cmd, err);
    }
}

#[derive(Debug)]
pub struct Rule {
    notification_templ: Option<Notification>,
    actions: Option<IndexMap<String, String>>,
    cmd: Option<String>,
    exec_in_dir: PathBuf,
}

impl Rule {
    pub fn new(
        notification_templ: Option<Notification>,
        cmd: Option<String>,
        actions: Option<IndexMap<String, String>>,
        exec_in_dir: PathBuf,
    ) -> Self {
        Self {
            notification_templ,
            cmd,
            actions,
            exec_in_dir,
        }
    }

    async fn notify(&self) {
        let template = match &self.notification_templ {
            Some(template) => template,
            None => return,
        };

        let handle = match template.show_async().await {
            Ok(handle) => handle,
            Err(err) => {
                eprintln!("Failed to show notification: {}", err);
                return;
            }
        };

        let actions = match self.actions.to_owned() {
            Some(actions) => actions,
            None => return,
        };

        let exec_in_dir = self.exec_in_dir.to_owned();

        unblock(|| {
            handle.wait_for_action(|action| {
                if let Some(cmd) = actions.get(action) {
                    smol::block_on(run_command(&exec_in_dir, cmd));
                };
            });
            (actions, exec_in_dir)
        })
        .await;
    }

    pub async fn execute(&self) {
        let cmd_fut = async {
            match self.cmd.as_ref() {
                Some(cmd) => {
                    let _: () = run_command(&self.exec_in_dir, cmd).await;
                    Some(())
                }
                None => None,
            }
        };

        let handle_fut = self.notify();

        zip(cmd_fut, handle_fut).await;
    }
}

pub type PercentageRules = HashMap<i8, Rule>;
