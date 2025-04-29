
use std::process::Command;

#[tauri::command]
fn run_php_in_tinker(path: String, code: String) -> Result<String, String> {
    // Run PHP code in artisan tinker
    let output = Command::new("php")
        .arg("artisan")
        .arg("tinker")
        .arg("--execute") // Add --execute to run a specific PHP command
        .arg(&code)  // The code entered by the user
        .current_dir(&path) // Laravel project path
        .output()
        .map_err(|e| format!("Failed to run tinker: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_php_in_tinker])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
