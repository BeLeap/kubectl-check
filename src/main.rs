use std::{
    env,
    io::{self, Write},
    process::Command,
};

use atty::Stream;
use colored::Colorize;

mod config;
mod error;

fn main() -> error::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if atty::is(Stream::Stdout) {
        let kube_config = config::kubeconfig::read()?;
        let metadata = extract_metadata(kube_config, &args)?;

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

#[derive(Clone, Debug)]
struct KubectlMetadata {
    target_context: String,
    target_namespace: String,
    command: String,
}

fn get_value(fragment: &str, prefix: &str, iter: &mut std::slice::Iter<String>) -> Option<String> {
    if fragment == prefix {
        let next_fragment = iter.next();
        next_fragment.map(|it| it.to_string())
    } else if fragment.starts_with(&format!("{}=", prefix)) {
        Some(
            fragment
                .replace(&format!("{}=", prefix), "")
                .trim()
                .to_string(),
        )
    } else {
        None
    }
}

fn extract_metadata(
    kube_config: config::kubeconfig::KubeConfig,
    args: &Vec<String>,
) -> error::Result<KubectlMetadata> {
    let mut context_from_command = None;
    let mut namespace_from_command = None;
    let mut command = None;

    let mut command_iter = args.iter();
    while let Some(fragment) = command_iter.next() {
        if fragment.starts_with("-") {
            if let Some(value) = get_value(fragment, "--context", &mut command_iter) {
                context_from_command = Some(value);
            }

            if let Some(value) = get_value(fragment, "--namespace", &mut command_iter) {
                namespace_from_command = Some(value)
            }
            if let Some(value) = get_value(fragment, "-n", &mut command_iter) {
                namespace_from_command = Some(value)
            }
        } else if command.is_none() {
            command = Some(fragment.to_string())
        }
    }

    let target_context = context_from_command.unwrap_or(kube_config.current_context);
    let target_namespace = namespace_from_command.unwrap_or(
        kube_config
            .contexts
            .iter()
            .find(|&context| context.name == target_context)
            .ok_or(error::Error::CurrentContextNotFound(target_context.clone()))?
            .context
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
    );

    Ok(KubectlMetadata {
        target_context,
        target_namespace,
        command: command.ok_or(error::Error::NoCommandSpecified)?,
    })
}
