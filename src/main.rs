use clap::{command, Arg, ArgAction};

fn main() {
    let matches = command!()
        .arg(Arg::new("command").required(true).action(ArgAction::Append))
        .get_matches();

    let command = matches
        .get_many::<String>("command")
        .unwrap_or_default()
        .collect::<Vec<_>>();

    println!("{:#?}", command)
}

struct KubeContext {
    cluster: String,
}

struct KubeContexts {
    contexts: Vec<KubeContext>,
}

struct KubeConfig {
    current_context: String,
    contexts: KubeContexts,
}

struct KubeMetadata {
    context: String,
    namespace: String,
}

fn extract_metadata(kube_config: KubeConfig, command: Vec<&String>) -> KubeMetadata {
    let context_from_command = command
        .iter()
        .position(|&fragment| fragment == "--context")
        .and_then(|index| command.get(index + 1).map(|it| it.to_string()));

    KubeMetadata {
        context: context_from_command.unwrap_or(kube_config.current_context),
        namespace: todo!(),
    }
}

#[cfg(test)]
mod tests {
    mod extract_metadata {
        use crate::{extract_metadata, KubeConfig, KubeContexts};

        #[test]
        fn it_should_get_context_from_command() {
            let kube_config = KubeConfig {
                current_context: "context-from-kube-config".to_string(),
                contexts: KubeContexts { contexts: vec![] },
            };
            let command = [
                "kubectl",
                "--context",
                "context-from-command",
                "get",
                "pods",
            ]
            .map(|it| it.to_string())
            .to_vec();
            let result = extract_metadata(kube_config, command.iter().collect());

            assert_eq!(result.context, "context-from-command");
        }

        #[test]
        fn it_should_get_context_from_kube_context_if_not_exists_in_command() {
            let kube_config = KubeConfig {
                current_context: "context-from-kube-config".to_string(),
                contexts: KubeContexts { contexts: vec![] },
            };
            let command = ["kubectl", "get", "pods"].map(|it| it.to_string()).to_vec();
            let result = extract_metadata(kube_config, command.iter().collect());

            assert_eq!(result.context, "context-from-kube-config");
        }
    }
}
