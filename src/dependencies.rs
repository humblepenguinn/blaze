use async_recursion::async_recursion;
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

use crate::utils::{copy_folder_contents, read_package_json};

#[derive(Deserialize, Serialize, Default)]
pub struct Dependencies {
    pub dependencies: std::collections::BTreeMap<String, String>,
}

impl Dependencies {
    pub fn from_package_json() -> Result<Dependencies, Box<dyn std::error::Error>> {
        let contents = read_package_json()?;
        let package_json: Dependencies = serde_json::from_str(&contents)?;
        Ok(package_json)
    }

    pub fn write_dependencies_to_package_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new("package.json").exists() {
            let mut file = File::create("package.json")?;
            let buffer = serde_json::to_value(&self.dependencies)?;
            file.write_all(serde_json::to_string_pretty(&buffer)?.as_bytes())?;
        }

        let mut file = File::open("package.json")?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut package_json: Value = serde_json::from_str(&contents)?;

        package_json["dependencies"] = serde_json::to_value(&self.dependencies)?;

        let mut file = File::create("package.json")?;
        file.write_all(serde_json::to_string_pretty(&package_json)?.as_bytes())?;

        Ok(())
    }

    pub async fn download_dependencies(&self) -> Result<(), String> {
        let mut all_dependencies = std::collections::BTreeMap::new();

        for dependency in self.dependencies.clone() {
            let name = dependency.0;
            let version_req = dependency.1.replace('~', "");
            let version_req = version_req.replace('^', "");

            let version = match get_version(&name, &version_req).await {
                Ok(version) => version,
                Err(error) => return Err(error),
            };

            let related_dependencies = match get_related_dependencies(
                name.clone(),
                version.clone(),
                self.dependencies.clone(),
            )
            .await
            {
                Ok(related_dependencies) => related_dependencies,
                Err(error) => {
                    println!(
                        "Could not get related dependencies for {}@{}: {}",
                        name, version, error
                    );
                    continue;
                }
            };

            for (name, version) in related_dependencies {
                all_dependencies.insert(name, version);
            }

            all_dependencies.insert(name.to_string(), version);
        }

        if !Path::new("node_modules").exists() {
            if let Err(e) = fs::create_dir("node_modules") {
                return Err(e.to_string());
            }
        }

        let mut download_tasks = Vec::new();
        let amount_of_dependencies = all_dependencies.len();

        let count = Arc::new(Mutex::new(0));

        let progress_count = count.clone();
        let progress_bar_thread = tokio::task::spawn(async move {
            let pb = ProgressBar::new(amount_of_dependencies as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );

            loop {
                let count = *progress_count.lock().unwrap();

                pb.set_position(count as u64);
                if count >= amount_of_dependencies {
                    pb.finish();
                    break;
                }

                time::sleep(Duration::from_millis(100)).await;
            }
        });

        for (name, mut version) in all_dependencies.clone() {
            let count = count.clone();
            version = version.replace('~', "");
            version = version.replace('^', "");

            if version.contains('*') {
                version = match get_latest_version(&name).await {
                    Ok(version) => version,
                    Err(error) => return Err(error),
                };
            }

            let download_task = tokio::task::spawn(async move {
                if let Err(e) = download_dependency(name, version).await {
                    println!("Could not download dependency: {}", e);
                }

                let mut count = count.lock().unwrap();
                *count += 1;
            });
            download_tasks.push(download_task);
        }

        for download_task in download_tasks {
            download_task.await.unwrap();
        }

        progress_bar_thread.await.unwrap();

        let mut extraction_tasks = Vec::new();

        for (name, _version) in all_dependencies {
            let extraction_task = tokio::task::spawn(extract_dependency(name.clone()));
            extraction_tasks.push(extraction_task);
        }

        for extraction_task in extraction_tasks {
            match extraction_task.await {
                Ok(_) => (),
                Err(error) => return Err(error.to_string()),
            };
        }

        Ok(())
    }
}

pub async fn get_latest_version(package_name: &str) -> Result<String, String> {
    let url = format!("https://registry.npmjs.org/{}", package_name);

    let response = match reqwest::get(&url).await {
        Ok(response) => response,
        Err(error) => return Err(error.to_string()),
    };

    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return Err(error.to_string()),
    };

    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(error) => return Err(error.to_string()),
    };

    Ok(String::from(json["dist-tags"]["latest"].as_str().unwrap()))
}

pub async fn get_version(package_name: &str, version_req: &str) -> Result<String, String> {
    let url = format!("https://registry.npmjs.org/{}", package_name);

    let response = match reqwest::get(&url).await {
        Ok(response) => response,
        Err(error) => return Err(error.to_string()),
    };

    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return Err(error.to_string()),
    };

    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(error) => return Err(error.to_string()),
    };

    let versions = match json["versions"].as_object() {
        Some(versions) => versions,
        None => return Err("Could not find versions".to_string()),
    };

    let version_req_semver = match Version::parse(version_req) {
        Ok(version_req_semver) => version_req_semver,
        Err(error) => return Err(error.to_string()),
    };

    let mut latest_version: Option<Version> = None;

    for version in versions {
        let version_str = version.0;
        let version = match Version::parse(version_str) {
            Ok(version) => version,
            Err(_) => continue,
        };

        if version.major == version_req_semver.major
            || ((version.minor == version_req_semver.minor) && version.major == 0)
            || ((version.patch == version_req_semver.patch)
                && version.major == 0
                && version.minor == 0)
        {
            match latest_version.clone() {
                Some(existing_version) => {
                    if version > existing_version {
                        latest_version = Some(version);
                    }
                }
                None => {
                    latest_version = Some(version);
                }
            }
        }
    }

    match latest_version {
        Some(version) => Ok(version.to_string()),
        None => Err("Could not find version".to_string()),
    }
}

#[async_recursion]
pub async fn get_related_dependencies(
    package_name: String,
    version: String,
    old_dependencies: std::collections::BTreeMap<String, String>,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let contents = read_package_json().unwrap();
    let new_dependencies: Dependencies = match serde_json::from_str(&contents) {
        Ok(dependencies) => dependencies,
        Err(error) => return Err(error.to_string()),
    };

    if Path::new("blaze.lock").exists() && (new_dependencies.dependencies == old_dependencies) {
        let mut lock_file = match File::open("blaze.lock") {
            Ok(lock_file) => lock_file,
            Err(error) => return Err(error.to_string()),
        };

        let mut buffer = Vec::new();

        lock_file.read_to_end(&mut buffer).unwrap();

        let deserialized_dependencies: std::collections::BTreeMap<String, String> =
            bincode::deserialize(&buffer).unwrap();

        return Ok(deserialized_dependencies);
    }

    let mut all_dependencies = std::collections::BTreeMap::new();

    let url = format!("https://registry.npmjs.org/{}/{}", package_name, version);

    let response = match reqwest::get(&url).await {
        Ok(response) => response,
        Err(error) => return Err(error.to_string()),
    };

    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return Err(error.to_string()),
    };

    let json: serde_json::Value = serde_json::from_str(&text).unwrap();

    let dependencies = match json["dependencies"].as_object() {
        Some(dependencies) => dependencies,
        None => return Ok(all_dependencies),
    };

    let mut handles = vec![];

    for dependency in dependencies {
        let dependency_name = dependency.0;
        let dependency_version = match dependency.1.as_str() {
            Some(version) => version,
            None => return Err("Could not get version".into()),
        };

        let cleaned_version = dependency_version.replace(&['^', '~'][..], "");

        let dependency_version = match get_version(dependency_name, &cleaned_version).await {
            Ok(version) => version,
            Err(error) => return Err(error),
        };

        all_dependencies.insert(dependency_name.to_string(), dependency_version.to_string());

        let handle = tokio::spawn(get_related_dependencies(
            dependency_name.clone(),
            dependency_version.to_string(),
            old_dependencies.clone(),
        ));
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        for (name, version) in result? {
            let version = version.replace(['^', '~'], "");
            all_dependencies.insert(name, version);
        }
    }

    let mut lock_file = File::create("blaze.lock").unwrap();

    match lock_file.write_all(&bincode::serialize(&all_dependencies).unwrap()) {
        Ok(_) => (),
        Err(error) => return Err(error.to_string()),
    };

    drop(lock_file);

    Ok(all_dependencies)
}

pub async fn download_dependency(
    package_name: String,
    version: String,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://registry.npmjs.org/{}/-/{}-{}.tgz",
        package_name, package_name, version
    );

    let client = reqwest::Client::new();
    let mut resp = client.get(url).send().await?;

    let mut file = File::create(format!("node_modules/{}-{}.tgz", package_name, version))?;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
    }

    Ok(())
}

fn find_tgz_file_with_word(directory: &str, target_word: &str) -> PathBuf {
    let matching_files = PathBuf::new();

    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.ends_with(".tgz") && file_name_str.contains(target_word) {
                    return entry.path();
                }
            }
        }
    }

    matching_files
}

pub async fn extract_dependency(package_name: String) -> Result<(), std::io::Error> {
    let path = find_tgz_file_with_word("node_modules", &package_name);
    let extraction_path = format!("node_modules/{}", package_name);

    if !Path::new(&extraction_path).exists() {
        fs::create_dir_all(&extraction_path)?;
    }

    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(File::open(&path)?));

    archive.unpack(&extraction_path)?;

    let pkg_path = format!("{}/package", extraction_path);
    let pkg_path = Path::new(&pkg_path);

    copy_folder_contents(pkg_path, &PathBuf::from(extraction_path))?;

    fs::remove_dir_all(pkg_path)?;
    fs::remove_file(&path)?;

    Ok(())
}
