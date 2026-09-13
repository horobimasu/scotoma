use std::path::PathBuf;

pub fn find_dir_from_items(start_path: &PathBuf, items: &[&str]) -> Option<PathBuf> {
    let mut current_dir = start_path
        .as_path();

    loop {
        let all_exist = items.iter().all(|item| {
            let full_path = current_dir
                .join(item);

            full_path.exists()
        });

        if all_exist {
            return Some(current_dir.to_path_buf());
        }

        current_dir = current_dir
            .parent()?;
    }
}
