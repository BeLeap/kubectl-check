use std::{
    env,
    error::Error,
    fmt, fs,
    io::{self, Write},
    process::{Command, ExitStatus},
};

use atty::Stream;
use colored::Colorize;
use yaml_rust2::YamlLoader;

enum KubectlCheckError {
    KubeconfigIo(io::Error),
    KubeconfigParse(yaml_rust2::ScanError),
    MalformedKubeconfig,
    CurrentContextNotFound(String),
    NoCommandSpecified,
    NotConfirmed,
    CommandFailed(ExitStatus),
}

impl fmt::Display for KubectlCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KubectlCheckError::KubeconfigIo(ref err) => {
                write!(f, "Could not read kubeconfig: {}", err)
            }
            KubectlCheckError::KubeconfigParse(ref err) => {
                write!(f, "Could not parse kubeconfig: {}", err)
            }
            KubectlCheckError::NotConfirmed => write!(f, "Execution cancelled."),
            KubectlCheckError::CommandFailed(status) => {
                write!(f, "{}", status)
            }
            KubectlCheckError::MalformedKubeconfig => write!(f, "Malformed kubeconfig"),
            KubectlCheckError::CurrentContextNotFound(current_context) => {
                write!(f, "Context not found: {}", current_context)
            }
            KubectlCheckError::NoCommandSpecified => write!(f, "No command for kubectl sepcified"),
        }
    }
}
impl fmt::Debug for KubectlCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl Error for KubectlCheckError {}

type KubectlCheckResult<T> = Result<T, KubectlCheckError>;

fn main() -> KubectlCheckResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if atty::is(Stream::Stdout) {
        let kube_config = read_kube_config()?;
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
                return Err(KubectlCheckError::NotConfirmed);
            }
        }
    }

    let mut command = Command::new("kubectl");
    command.args(args);

    let status = command.status().expect("could not execute kubectl");

    if status.success() {
        return Ok(());
    }

    return Err(KubectlCheckError::CommandFailed(status));
}

#[derive(Clone, Debug)]
struct KubeContextMetadata {
    namespace: Option<String>,
}

#[derive(Clone, Debug)]
struct KubeContext {
    name: String,
    context: KubeContextMetadata,
}

#[derive(Clone, Debug)]
struct KubeConfig {
    current_context: String,
    contexts: Vec<KubeContext>,
}

#[derive(Clone, Debug)]
struct KubectlMetadata {
    target_context: String,
    target_namespace: String,
    command: String,
}

fn extract_metadata(
    kube_config: KubeConfig,
    args: &Vec<String>,
) -> KubectlCheckResult<KubectlMetadata> {
    let mut context_from_command = None;
    let mut namespace_from_command = None;
    let mut command = None;

    let mut command_iter = args.iter();
    while let Some(fragment) = command_iter.next() {
        if fragment.starts_with("-") {
            let next_fragment = command_iter.next();
            if fragment == "--context" {
                context_from_command = next_fragment.map(|it| it.to_string());
            }

            if fragment == "--namespace" || fragment == "-n" {
                namespace_from_command = next_fragment.map(|it| it.to_string());
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
            .ok_or(KubectlCheckError::CurrentContextNotFound(
                target_context.clone(),
            ))?
            .context
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
    );

    Ok(KubectlMetadata {
        target_context,
        target_namespace,
        command: command.ok_or(KubectlCheckError::NoCommandSpecified)?,
    })
}

fn read_kube_config() -> KubectlCheckResult<KubeConfig> {
    let path = env::var("KUBECONFIG").unwrap_or(format!(
        "{}/.kube/config",
        env::var("HOME").unwrap_or("~".to_string())
    ));
    let contents = fs::read_to_string(path).map_err(|err| KubectlCheckError::KubeconfigIo(err))?;

    let documents = YamlLoader::load_from_str(&contents)
        .map_err(|err| KubectlCheckError::KubeconfigParse(err))?;
    let document = &documents[0];

    let contexts = &document["contexts"]
        .clone()
        .into_iter()
        .map(|context| {
            Ok(KubeContext {
                name: context["name"]
                    .as_str()
                    .ok_or(KubectlCheckError::MalformedKubeconfig)?
                    .to_string(),
                context: KubeContextMetadata {
                    namespace: context["context"]["namespace"]
                        .as_str()
                        .map(|it| it.to_string()),
                },
            })
        })
        .collect::<KubectlCheckResult<Vec<KubeContext>>>()?;

    Ok(KubeConfig {
        current_context: document["current-context"]
            .as_str()
            .ok_or(KubectlCheckError::MalformedKubeconfig)?
            .to_string(),
        contexts: contexts.clone(),
    })
}

#[cfg(test)]
mod tests {
    mod extract_metadata {
        use crate::{extract_metadata, KubeConfig, KubeContext, KubeContextMetadata};

        #[test]
        fn it_should_get_metadata_scenario_1() {
            let kube_config = KubeConfig {
                current_context: "context-from-kube-config".to_string(),
                contexts: vec![KubeContext {
                    name: "context-from-command".to_string(),
                    context: KubeContextMetadata {
                        namespace: Some("namespace-from-kube-config".to_string()),
                    },
                }],
            };
            let args = [
                "kubectl",
                "--context",
                "context-from-command",
                "get",
                "pods",
            ]
            .map(|it| it.to_string())
            .to_vec();
            let result = extract_metadata(kube_config, &args).unwrap();

            assert_eq!(result.target_context, "context-from-command");
            assert_eq!(result.target_namespace, "namespace-from-kube-config");
        }

        #[test]
        fn it_should_get_metadata_scenario_2() {
            let kube_config = KubeConfig {
                current_context: "context-from-kube-config".to_string(),
                contexts: vec![KubeContext {
                    name: "context-from-kube-config".to_string(),
                    context: KubeContextMetadata {
                        namespace: Some("namespace-from-kube-config".to_string()),
                    },
                }],
            };
            let args = ["kubectl", "get", "pods"].map(|it| it.to_string()).to_vec();
            let result = extract_metadata(kube_config, &args).unwrap();

            assert_eq!(result.target_context, "context-from-kube-config");
            assert_eq!(result.target_namespace, "namespace-from-kube-config");
        }
    }
}
