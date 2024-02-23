use std::{
    fs::{self, metadata},
    io::Error,
};

use regex::Regex;

pub trait DependencyReader {
    fn read_unusued_deps(&mut self) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NpmReader {
    pub dependencies: Vec<String>,
    pub unusued_dependencies: Vec<String>,
    root_path: String,
}

impl NpmReader {
    pub fn new(path: String) -> Result<NpmReader, Error> {
        if metadata(&path).unwrap().is_file() {
            panic!("Path must be a directory!")
        }
        Ok(NpmReader {
            root_path: path,
            dependencies: Vec::new(),
            unusued_dependencies: Vec::new(),
        })
    }

    fn find_mismatches(all_deps: Vec<String>, found_deps: Vec<String>) -> Vec<String> {
        let mut mismatches: Vec<String> = Vec::new();

        for item in all_deps.iter() {
            match found_deps.contains(item) {
                false => mismatches.push(item.to_string()),
                _ => (),
            }
        }
        mismatches
    }

    fn read_installed_deps(&mut self) -> Result<(), Error> {
        let path = self.root_path.as_str();
        let package_json_content =
            fs::read_to_string(format!("{path}/package.json")).expect("Failed to read file");

        // TODO: devDependencies are not captured
        let re = Regex::new(r#""(?:dependencies|devDependencies)"\s*:\s*\{([^}]*)\}"#)
            .expect("Failed to compile regex");

        if let Some(capture) = re.captures(&package_json_content) {
            let dependencies_content = capture.get(1).unwrap().as_str();

            let re_dependency =
                Regex::new(r#""([^"]+)"\s*:\s*"(.*?[^\\])""#).expect("Failed to compile regex");

            re_dependency
                .captures_iter(dependencies_content)
                .for_each(|dependency_capture| {
                    let dependency_name = dependency_capture.get(1).unwrap().as_str();
                    self.dependencies.push(dependency_name.to_string());
                });
        }
        Ok(())
    }
}

impl DependencyReader for NpmReader {
    fn read_unusued_deps(&mut self) -> Result<(), Error> {
        self.read_installed_deps()?;
        let root = &self.root_path;

        let js_files_content = read_js_files_in_dir(&root);

        let index_content = js_files_content?[0].clone();

        let re = Regex::new(r#"import\s+[^'"]+\s+from\s+['"]([^'"]+)['"];"#)
            .expect("Failed to compile regex");

        let found = find_matches(re, &index_content)?;

        for deps in self.dependencies.iter() {
            match found.contains(&deps) {
                false => self.unusued_dependencies.push(deps.to_string()),
                _ => (),
            }
        }
        self.unusued_dependencies = NpmReader::find_mismatches(self.dependencies.to_owned(), found);
        Ok(())
    }
}

fn read_js_files_in_dir(dir: &str) -> Result<Vec<String>, std::io::Error> {
    let dir_entries = fs::read_dir(&dir)?;

    let js_files_conent: Result<Vec<String>, Error> = dir_entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if let Some(extension) = path.extension() {
                if extension != "js" {
                    return None;
                }
                let content = fs::read_to_string(&path).unwrap();
                return Some(Ok(content.trim().to_string()));
            }
            None
        })
        .collect();
    js_files_conent
}

fn find_matches(regex: Regex, content: &str) -> Result<Vec<String>, Error> {
    let mut findings: Vec<String> = Vec::new();
    for capture in regex.captures_iter(content) {
        let found = capture.get(1).unwrap().as_str();
        findings.push(found.to_string());
    }
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, fs, path::PathBuf};

    use crate::{read_js_files_in_dir, DependencyReader, NpmReader};

    struct MockFile {
        temp_dir: PathBuf,
        file: PathBuf,
    }

    #[test]
    fn should_read_installed_dependencies() -> Result<(), std::io::Error> {
        let mut reader = setup_npm_reader(vec![String::from("test-dependency")])?;
        reader.read_installed_deps()?;
        assert_eq!(reader.dependencies, vec!["test-dependency"]);

        Ok(())
    }

    #[test]
    fn should_read_unused_deps() -> Result<(), std::io::Error> {
        let mut reader = setup_npm_reader(vec![
            String::from("test-dependency"),
            String::from("vimbtw"),
        ])?;
        let _ = setup_import_statements_file(vec![String::from("test-dependency")])?;
        reader.read_unusued_deps()?;
        assert_eq!(reader.unusued_dependencies, vec!["vimbtw"]);

        Ok(())
    }

    #[test]
    fn should_read_js_files_content() -> Result<(), std::io::Error> {
        let mock = setup_import_statements_file(vec![String::from("vimbtw")])?;
        let js_contents =
            read_js_files_in_dir(mock.temp_dir.as_path().as_os_str().to_str().unwrap())?;
        assert_eq!(js_contents, vec!["import * from vimbtw"]);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn it_fails_when_path_is_invalid() {
        let _ = NpmReader::new("/Users/my_user/repo".to_string());
    }

    #[test]
    #[ignore = "todo"]
    fn it_works_when_root_is_directory() {
        todo!()
    }

    #[test]
    #[ignore = "todo"]
    fn reads_dependencies() {
        todo!()
    }

    fn setup_npm_reader(deps: Vec<String>) -> Result<NpmReader, std::io::Error> {
        let mock = setup_package_json_file(deps)?;
        let dir = mock.temp_dir.into_os_string().into_string().unwrap();
        Ok(NpmReader::new(dir).unwrap())
    }

    fn setup_package_json_file(deps: Vec<String>) -> Result<MockFile, std::io::Error> {
        let temp_dir = temp_dir();

        let mut dependencies_content = String::new();
        for dep in deps {
            dependencies_content.push_str(&format!("\"{}\": \"1.0.0\",\n", dep));
        }

        let package_json_content = format!(
            r#"{{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {{
                {}
            }}
        }}"#,
            dependencies_content
        );

        let path_to_package_json = temp_dir.as_path().join("package.json");
        fs::write(&path_to_package_json, package_json_content).expect("should have written file.");
        Ok(MockFile {
            file: path_to_package_json,
            temp_dir,
        })
    }

    fn setup_import_statements_file(
        imported_deps: Vec<String>,
    ) -> Result<MockFile, std::io::Error> {
        let temp_dir = temp_dir();

        let mut file_content = format!(
            r#"
            "#,
        );
        for ele in imported_deps {
            file_content.push_str(&format!("import * from {}", ele.as_str()));
        }

        let path_to_js_file = temp_dir.as_path().join("main.js");
        fs::write(&path_to_js_file, file_content).expect("should have written file.");
        Ok(MockFile {
            file: path_to_js_file,
            temp_dir,
        })
    }
}
