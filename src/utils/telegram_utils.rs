pub fn escape_markdown_v2_reversed_chars(input: &str) -> String {
    let mut output = String::new();
    for c in input.chars() {
        match c {
            '[' | ']' | '(' | ')' | '~' | '>' | '#' | '+' | '-' | '=' | '|' | '{' | '}' | '.'
            | '!' => {
                output.push('\\');
                output.push(c);
            }
            _ => output.push(c),
        }
    }
    output
}
