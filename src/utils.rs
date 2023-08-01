use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

pub fn copy_folder_contents(source: &Path, destination: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;

        let file_type = entry.file_type()?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry_path.file_name().unwrap());

        if file_type.is_dir() {
            fs::create_dir(&destination_path)?;
            copy_folder_contents(&entry_path, &destination_path)?;
        } else {
            fs::copy(&entry_path, &destination_path)?;
        }
    }

    Ok(())
}

pub fn read_package_json() -> Result<String, std::io::Error> {
    let mut file = File::open("package.json")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
