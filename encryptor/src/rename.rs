use std::sync::Mutex;

static COUNTER: Mutex<String> = Mutex::new(String::new());

pub fn get_new_name() -> String {
    let mut counter = COUNTER
        .lock()
        .unwrap();

    if counter.is_empty() {
        *counter = String::from("a");
        return counter.clone();
    }

    let mut characters = counter
        .chars()
        .collect::<Vec<char>>();

    let mut index = characters.len() - 1;

    loop {
        if characters[index] == 'z' {
            characters[index] = 'a';

            if index == 0 {
                characters.insert(0, 'a');
                break;
            }

            index -= 1;
        } else {
            characters[index] = (characters[index] as u8 + 1) as char;
            break;
        }
    }

    *counter = characters
        .into_iter()
        .collect();

    counter.clone()
}
