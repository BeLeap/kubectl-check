use crate::error;

use super::kubeconfig;

#[derive(Clone, Debug)]
pub struct KubectlMetadata {
    pub target_context: String,
    pub target_namespace: String,
    pub command: String,
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

pub fn from(
    kube_config: kubeconfig::KubeConfig,
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
