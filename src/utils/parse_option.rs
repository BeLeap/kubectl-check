pub fn get_value(
    fragment: &str,
    prefix: &str,
    iter: &mut std::slice::Iter<String>,
) -> Option<String> {
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
