use bitcoin::{Network, NetworkKind, PrivateKey, PublicKey, key::Secp256k1};
use esp_idf_svc::sys::{esp_err_t, nvs_flash_init, nvs_flash_init_partition};
use esp_idf_svc::sys::{
    nvs_close, nvs_commit, nvs_get_str, nvs_handle_t, nvs_open, nvs_open_mode_t, nvs_set_str,
};

use crate::SecretKey;
//TODO: add security schemes to NVS
pub fn initialize_nvs() -> Result<(), esp_err_t> {
    let partition_label = std::ffi::CString::new("nvs").unwrap();
    let result = unsafe { nvs_flash_init_partition(partition_label.as_ptr()) };
    if result == 0 {
        Ok(())
    } else {
        Err(result)
    }
}

pub fn open_nvs_partition() -> Result<nvs_handle_t, esp_err_t> {
    let partition_label = std::ffi::CString::new("nvs").unwrap();
    let mut handle: nvs_handle_t = 0;
    let result = unsafe { nvs_open(partition_label.as_ptr(), 1, &mut handle) };
    if result == 0 {
        Ok(handle)
    } else {
        Err(result)
    }
}

pub fn save_value(handle: nvs_handle_t, key: &str, value: &str) -> Result<(), esp_err_t> {
    let key_cstr = std::ffi::CString::new(key).unwrap();
    let value_cstr = std::ffi::CString::new(value).unwrap();
    let result = unsafe { nvs_set_str(handle, key_cstr.as_ptr(), value_cstr.as_ptr()) };
    if result == 0 {
        let commit_result = unsafe { nvs_commit(handle) };
        if commit_result == 0 {
            Ok(())
        } else {
            Err(commit_result)
        }
    } else {
        Err(result)
    }
}

pub fn get_value(handle: nvs_handle_t, key: &str) -> Result<String, esp_err_t> {
    let key_cstr = std::ffi::CString::new(key).unwrap();
    let mut buffer_len: usize = 0;
    let result = unsafe {
        nvs_get_str(
            handle,
            key_cstr.as_ptr(),
            std::ptr::null_mut(),
            &mut buffer_len,
        )
    };
    if result != 0 {
        return Err(result);
    }
    let mut buffer: Vec<u8> = vec![0; buffer_len];
    let result = unsafe {
        nvs_get_str(
            handle,
            key_cstr.as_ptr(),
            buffer.as_mut_ptr() as *mut i8,
            &mut buffer_len,
        )
    };
    if result == 0 {
        buffer.pop(); // Remove the null terminator
        Ok(String::from_utf8(buffer).unwrap())
    } else {
        Err(result)
    }
}

pub fn close_nvs_partition(handle: nvs_handle_t) {
    unsafe { nvs_close(handle) };
}

pub fn save_bitcoin_private_key(
    handle: nvs_handle_t,
    private_key: PrivateKey,
) -> Result<(), esp_err_t> {
    // Parse the private key

    // Convert the private key to WIF format
    let wif = private_key.to_wif();

    // Extract the first 6 characters
    let first_six_chars = &wif[..6];
    println!("First 6 characters: {}", first_six_chars);

    // Remove the first 6 characters
    let trimmed_wif = &wif[6..];

    println!("Trimmed WIF: {}", trimmed_wif);
    // Save the trimmed WIF to NVS
    save_value(handle, "bitcoin_private_key", trimmed_wif)
}

pub fn nvs_example() {
    match initialize_nvs() {
        Ok(_) => println!("NVS initialized successfully"),
        Err(err) => {
            eprintln!("Failed to initialize NVS: {}", err);
            return;
        }
    }

    let handle = match open_nvs_partition() {
        Ok(handle) => handle,
        Err(err) => {
            eprintln!("Failed to open NVS partition: {}", err);
            return;
        }
    };

    let key = "example_key";
    let value = "example_value";

    match save_value(handle, key, value) {
        Ok(_) => println!("Value saved successfully"),
        Err(err) => {
            eprintln!("Failed to save value: {}", err);
            close_nvs_partition(handle);
            return;
        }
    }

    match get_value(handle, key) {
        Ok(retrieved_value) => println!("Retrieved value: {}", retrieved_value),
        Err(err) => eprintln!("Failed to retrieve value: {}", err),
    }

    let private_key: PrivateKey;
    let hex_key = "3d7eee92f66b5a9c95b0c213d4a4286105d650b39c9e1bc86c0c0c34972370f7";
    match hex::decode(hex_key) {
        Ok(decoded) => {
            match SecretKey::from_slice(&decoded) {
                Ok(secret_key) => {
                    let secp = Secp256k1::new();
                        private_key = PrivateKey {
                        compressed: true,
                        network: NetworkKind::Main,
                        inner: secret_key,
                    };
                    println!("Private key: {:?}", private_key);
                    match save_bitcoin_private_key(handle, private_key) {
                        Ok(_) => println!("Bitcoin private key saved successfully"),
                        Err(err) => eprintln!("Failed to save Bitcoin private key: {}", err),
                    }
                }
                Err(err) => eprintln!("Failed to create secret key: {}", err),
            }
        }
        Err(err) => eprintln!("Failed to decode hex key: {}", err),
    }
    

    close_nvs_partition(handle);
}
