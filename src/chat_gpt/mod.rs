pub use grammar_checker::check_grammar;
pub use translation::translate;
pub use variable_namer::naming_variable;

mod grammar_checker;
mod translation;
mod variable_namer;

fn split_options_and_body(
    user_input: String,
    default: String,
    option_flag: char,
) -> (String, String) {
    let parts = user_input.splitn(2, ' ').collect::<Vec<&str>>();
    if parts.len() == 1 {
        return (default, parts[0].to_string());
    }

    let lang = parts[0];
    if let Some(lang) = lang.strip_prefix(&format!("-{option_flag}")) {
        if lang.is_empty() {
            return (default, parts[1].to_string());
        }
        return (lang.to_string(), parts[1].to_string());
    }

    (default, user_input)
}
