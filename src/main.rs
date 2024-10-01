use std::{env, fs};

use yaml_rust2::YamlLoader;

fn main() {
    let command = std::env::args().collect();

    let kube_config = read_kube_config();
    let metadata = extract_metadata(kube_config, command);

    println!("{:#?}", metadata)
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
struct KubeMetadata {
    current_context: String,
    current_namespace: String,
}

fn extract_metadata(kube_config: KubeConfig, command: Vec<String>) -> KubeMetadata {
    let mut context_from_command = None;
    let mut namespace_from_command = None;

    let mut command_iter = command.iter();
    while let Some(fragment) = command_iter.next() {
        if fragment == "--context" {
            context_from_command = command_iter.next().map(|it| it.to_string());
        }

        if fragment == "--namespace" {
            namespace_from_command = command_iter.next().map(|it| it.to_string());
        }
    }

    let current_context = context_from_command.unwrap_or(kube_config.current_context);
    let current_namespace = namespace_from_command.unwrap_or(
        kube_config
            .contexts
            .iter()
            .find(|&context| context.name == current_context)
            .expect("Malformed kubeconfig current context not found!!")
            .context
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
    );

    KubeMetadata {
        current_context,
        current_namespace,
    }
}

fn read_kube_config() -> KubeConfig {
    let path = env::var("KUBECONFIG").unwrap_or(format!(
        "{}/.kube/config",
        env::var("HOME").unwrap_or("~".to_string())
    ));
    let contents = fs::read_to_string(path).expect("could not read kubeconfig");

    let documents = YamlLoader::load_from_str(&contents).unwrap();
    let document = &documents[0];

    let contexts = &document["contexts"]
        .clone()
        .into_iter()
        .map(|context| KubeContext {
            name: context["name"].as_str().unwrap().to_string(),
            context: KubeContextMetadata {
                namespace: context["context"]["namespace"]
                    .as_str()
                    .map(|it| it.to_string()),
            },
        })
        .collect::<Vec<_>>();

    KubeConfig {
        current_context: document["current_context"].as_str().unwrap().to_string(),
        contexts: contexts.clone(),
    }
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

            assert_eq!(result.current_context, "context-from-command");
            assert_eq!(result.current_namespace, "namespace-from-kube-config");
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
            let command = ["kubectl", "get", "pods"].map(|it| it.to_string()).to_vec();
            let result = extract_metadata(kube_config, command.iter().collect());

            assert_eq!(result.current_context, "context-from-kube-config");
            assert_eq!(result.current_namespace, "namespace-from-kube-config");
        }
    }
}
