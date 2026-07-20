use std::path::Path;
use std::process::Command;

#[derive(Debug, serde::Deserialize)]
struct RunPayload {
    path: String,
    code: String,
    mode: String,
    container_name: Option<String>,
    container_path: Option<String>,
}

#[tauri::command]
async fn pick_laravel_directory() -> Result<String, String> {
    let candidates = [
        ("zenity", vec!["--file-selection", "--directory", "--title=Select Laravel project folder"]),
        ("kdialog", vec!["--getexistingdirectory"]),
    ];

    for (program, args) in candidates {
        let output = Command::new(program).args(&args).output();
        match output {
            Ok(result) if result.status.success() => {
                let path = String::from_utf8_lossy(&result.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }

    Err("Unable to open a folder picker. Install zenity or kdialog, or enter the Laravel path manually.".to_string())
}

#[tauri::command]
fn list_docker_containers() -> Result<Vec<String>, String> {
    let output = Command::new("docker")
        .args(["ps", "--format", "{{.Names}}"])
        .output()
        .map_err(|error| format!("Unable to query Docker: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Docker is not available on this machine.".to_string()
        } else {
            stderr
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let containers = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    Ok(containers)
}

#[tauri::command]
fn run_php_in_tinker(payload: RunPayload) -> Result<String, String> {
    let trimmed_code = payload.code.trim();
    if trimmed_code.is_empty() {
        return Err("Please provide PHP code to execute.".into());
    }

    match payload.mode.as_str() {
        "docker" => run_in_docker(&payload, trimmed_code),
        _ => run_locally(&payload, trimmed_code),
    }
}

fn run_locally(payload: &RunPayload, code: &str) -> Result<String, String> {
    let trimmed_path = payload.path.trim();
    if trimmed_path.is_empty() {
        return Err("Please provide a Laravel project path.".into());
    }

    let project_dir = Path::new(trimmed_path);
    let artisan_path = project_dir.join("artisan");

    if !artisan_path.exists() {
        return Err(format!(
            "No Laravel project was found at '{}'. Expected an artisan file.",
            trimmed_path
        ));
    }

    let php_binary = std::env::var("PHP_BINARY").unwrap_or_else(|_| "php".to_string());
    let output = Command::new(&php_binary)
        .args(["artisan", "tinker", "--execute", code])
        .current_dir(project_dir)
        .output()
        .map_err(|error| format!("Failed to launch Tinker: {error}"))?;

    handle_command_output(output)
}

fn run_in_docker(payload: &RunPayload, code: &str) -> Result<String, String> {
    let container_name = payload.container_name.as_deref().unwrap_or_default().trim();
    let container_path = payload.container_path.as_deref().unwrap_or_default().trim();

    if container_name.is_empty() {
        return Err("Please provide a Docker container name.".into());
    }

    if container_path.is_empty() {
        return Err("Please provide the project path inside the container.".into());
    }

    let output = Command::new("docker")
        .args([
            "exec",
            "-i",
            "-w",
            container_path,
            container_name,
            "php",
            "artisan",
            "tinker",
            "--execute",
            code,
        ])
        .output()
        .map_err(|error| format!("Failed to execute inside Docker: {error}"))?;

    handle_command_output(output)
}

fn handle_command_output(output: std::process::Output) -> Result<String, String> {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if stdout.is_empty() {
            "Command completed successfully.".to_string()
        } else {
            stdout
        })
    } else {
        let combined = if stdout.is_empty() {
            stderr
        } else if stderr.is_empty() {
            stdout
        } else {
            format!("{stderr}\n{stdout}")
        };

        Err(if combined.is_empty() {
            format!("Tinker exited with status {}.", output.status)
        } else {
            combined
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![pick_laravel_directory, list_docker_containers, run_php_in_tinker])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
