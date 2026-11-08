use keyring::Entry;

const SERVICE_NAME: &str = "com.storyquest.studio.credentials";

pub fn store_api_key(credential_key_name: &str, secret_api_key: &str) -> Result<(), String> {
    if secret_api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }
    let entry = Entry::new(SERVICE_NAME, credential_key_name)
        .map_err(|e| format!("Failed to initialize credential store entry: {}", e))?;
    entry
        .set_password(secret_api_key)
        .map_err(|e| format!("Failed to save API key to Windows Credential Manager: {}", e))
}

pub fn retrieve_api_key(credential_key_name: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, credential_key_name)
        .map_err(|e| format!("Failed to access credential store: {}", e))?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Credential lookup error: {}", e)),
    }
}

pub fn delete_api_key(credential_key_name: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, credential_key_name)
        .map_err(|e| format!("Failed to access credential store: {}", e))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Failed to delete credential: {}", e)),
    }
}
