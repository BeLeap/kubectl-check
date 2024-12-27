use crate::error;

use super::{kubeconfig, opts};

#[derive(Clone, Debug)]
pub struct KubectlMetadata {
    pub target_context: String,
    pub target_namespace: String,
    pub first_command: String,
}

pub fn from(
    kube_config: kubeconfig::KubeConfig,
    args: &Vec<String>,
) -> error::Result<KubectlMetadata> {
    let opts = opts::from_args(args);

    let target_context = opts.context.unwrap_or(kube_config.current_context);
    let target_namespace = opts.namespace.unwrap_or(
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
        first_command: opts.first_command.ok_or(error::Error::NoCommandSpecified)?,
    })
}
