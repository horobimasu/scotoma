use std::env;

pub fn expand_env_vars(path: &str) -> String {
    let mut result = String::new();

    let mut chars = path
        .chars()
        .peekable();

    while let Some(char) = chars.next() {
        if char != '%' {
            result.push(char);
            continue;
        }

        let mut name = String::new();
        let mut closed = false;

        while let Some(char) = chars.next() {
            if char == '%' {
                closed = true;
                break;
            }

            name.push(char);
        }

        if closed && !name.is_empty() {
            match env::var(&name) {
                Ok(val) => result.push_str(&val),
                Err(_) => {
                    result.push('%');
                    result.push_str(&name);
                    result.push('%');
                }
            }
        } else {
            result.push('%');
            result.push_str(&name);
        }
    }

    result
}

#[test]
fn test_expand_env_vars() {
    println!("userprofile - {}", expand_env_vars("%USERPROFILE%\\Desktop"));
    println!("system root - {}", expand_env_vars("%SYSTEMROOT%"));
}
