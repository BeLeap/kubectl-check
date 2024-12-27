pub struct Opts {
    pub context: Option<String>,
    pub namespace: Option<String>,

    pub first_command: Option<String>,
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

pub fn from_args(args: &Vec<String>) -> Opts {
    let mut context = None;
    let mut namespace = None;
    let mut first_command = None;

    let mut arg_it = args.iter();
    while let Some(fragment) = arg_it.next() {
        if fragment.starts_with("-") {
            if let Some(value) = get_value(fragment, "--context", &mut arg_it) {
                context = Some(value);
            }

            if let Some(value) = get_value(fragment, "--namespace", &mut arg_it) {
                namespace = Some(value)
            }
            if let Some(value) = get_value(fragment, "-n", &mut arg_it) {
                namespace = Some(value)
            }
        } else if first_command.is_none() {
            first_command = Some(fragment.to_string())
        }
    }
    return Opts {
        context,
        namespace,
        first_command,
    };
}
