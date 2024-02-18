use std::{
    fs::{self, metadata},
    io::Error,
};

use regex::Regex;

fn main() -> Result<(), Error> {
    let path = std::env::args().nth(1).expect("no file given");
    let mut reader = NpmReader::new(path)?;
    reader.read_installed_deps();
    println!("Found dependencies: {:?}", reader.dependencies);
    reader.read_unusued_deps();
    println!("Found unused deps: {:?}", reader.unusued_dependencies);
    Ok(())
}

trait DependencyReader {
    fn read_installed_deps(&mut self) -> Result<(), Error>;
    fn read_unusued_deps(&mut self) -> Result<(), Error>;
}

#[derive(Debug)]
struct NpmReader {
    dependencies: Vec<String>,
    unusued_dependencies: Vec<String>,
    root_path: String,
}

impl NpmReader {
    fn new(path: String) -> Result<NpmReader, Error> {
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
}

impl DependencyReader for NpmReader {
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

    fn read_unusued_deps(&mut self) -> Result<(), Error> {
        let root = &self.root_path;
        let index_file = format!("{root}/index.js");
        let index_content = fs::read_to_string(index_file).expect("Failed to read index.js");

        let re = Regex::new(r#"import\s+[^'"]+\s+from\s+['"]([^'"]+)['"];"#)
            .expect("Failed to compile regex");

        let found = find_matches(re, &index_content)?;

        for deps in self.dependencies.iter() {
            match found.contains(&deps) {
                false => self.unusued_dependencies.push(deps.to_string()),
                _ => (),
            }
        }

        Ok(self.unusued_dependencies = NpmReader::find_mismatches(self.dependencies.to_owned(), found))
    }
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
    use crate::NpmReader;

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
}
