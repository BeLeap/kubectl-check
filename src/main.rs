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

fn get_context(command: Vec<&String>) -> Option<String> {
    return command
        .iter()
        .position(|&fragment| fragment == "--context")
        .and_then(|index| command.get(index + 1).map(|it| it.to_string()));
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}

    mod get_context {
        use crate::get_context;

        #[test]
        fn it_should_get_context_from_command() {
            let command = ["kubectl", "--context", "test", "get", "pods"]
                .map(|it| it.to_string())
                .to_vec();
            let result = get_context(command.iter().collect());

            assert_eq!(result.unwrap(), "test");
        }

        #[test]
        fn it_should_get_context_from_kube_context_if_not_exists_in_command() {
            let command = ["kubectl", "get", "pods"].map(|it| it.to_string()).to_vec();
            let result = get_context(command.iter().collect());

            assert_eq!(result.unwrap(), "test");
        }
    }
}
