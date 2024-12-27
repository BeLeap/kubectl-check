use std::{
    env,
    io::{self, Write},
    process::Command,
};

use atty::Stream;
use colored::Colorize;

mod config;
mod error;
mod utils;

fn main() -> error::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if atty::is(Stream::Stdout) {
        let kube_config = config::kubeconfig::read()?;
        let metadata = config::metadata::from(kube_config, &args)?;

        let unsafe_command_list_env =
            env::var("KUBECTL_CHECK_UNSAFE").unwrap_or_else(|_| "".to_string());
        let unsafe_command_list = if unsafe_command_list_env.is_empty() {
            vec![
                "edit", "delete", "rollout", "scale", "cordon", "uncordon", "drain", "taint",
                "exec", "create", "apply",
            ]
        } else {
            unsafe_command_list_env.split(",").collect()
        };

        if unsafe_command_list.contains(&metadata.command.as_str()) {
            print!(
                "Running {} over {}({}) (Y/n): ",
                metadata.command.as_str().red().bold(),
                metadata.target_context.as_str().green(),
                metadata.target_namespace.as_str().green(),
            );
            io::stdout().flush().expect("could not flush stdout");

            let stdin = io::stdin();
            let mut buffer = String::new();
            if let Err(e) = stdin.read_line(&mut buffer) {
                panic!("{}", e);
            };

            if buffer.trim() != "Y" {
                return Err(error::Error::NotConfirmed);
            }
        }
    }

    let mut command = Command::new("kubectl");
    command.args(args);

    let status = command.status().expect("could not execute kubectl");

    if status.success() {
        return Ok(());
    }

    return Err(error::Error::CommandFailed(status));
}
