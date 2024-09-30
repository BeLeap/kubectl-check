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

fn get_context(command: Vec<&String>) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}

    mod get_context {
        use crate::get_context;

        #[test]
        fn it_should_get_context_from_command() {
            let result = get_context(vec![
                &"kubectl".to_string(),
                &"--context".to_string(),
                &"test".to_string(),
            ]);

            assert_eq!(result, "test");
        }
    }
}
