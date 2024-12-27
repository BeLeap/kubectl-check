use std::{env, fs};

use yaml_rust2::YamlLoader;

use crate::error;

#[derive(Clone, Debug)]
pub struct KubeContextMetadata {
    pub namespace: Option<String>,
}

#[derive(Clone, Debug)]
pub struct KubeContext {
    pub name: String,
    pub context: KubeContextMetadata,
}
#[derive(Clone, Debug)]
pub struct KubeConfig {
    pub current_context: String,
    pub contexts: Vec<KubeContext>,
}

pub fn read() -> error::Result<KubeConfig> {
    let path = env::var("KUBECONFIG").unwrap_or(format!(
        "{}/.kube/config",
        env::var("HOME").unwrap_or("~".to_string())
    ));
    let contents = fs::read_to_string(path).map_err(|err| error::Error::KubeconfigIo(err))?;

    let documents =
        YamlLoader::load_from_str(&contents).map_err(|err| error::Error::KubeconfigParse(err))?;
    let document = &documents[0];

    let contexts = &document["contexts"]
        .clone()
        .into_iter()
        .map(|context| {
            Ok(KubeContext {
                name: context["name"]
                    .as_str()
                    .ok_or(error::Error::MalformedKubeconfig)?
                    .to_string(),
                context: KubeContextMetadata {
                    namespace: context["context"]["namespace"]
                        .as_str()
                        .map(|it| it.to_string()),
                },
            })
        })
        .collect::<error::Result<Vec<KubeContext>>>()?;

    Ok(KubeConfig {
        current_context: document["current-context"]
            .as_str()
            .ok_or(error::Error::MalformedKubeconfig)?
            .to_string(),
        contexts: contexts.clone(),
    })
}
