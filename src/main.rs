use std::fs::{self};

use regex::Regex;

fn main() {
    let path = std::env::args().nth(1).expect("no file given");
    let mut reader = NpmReader::default();
    reader.read(&path);
    println!("Found dependencies: {:?}", reader.dependencies);
}

trait DependencyReader {
    fn read(&mut self, path: &str);
}

#[derive(Debug, Default)]
struct NpmReader {
    dependencies: Vec<String>,
}
impl DependencyReader for NpmReader {
    fn read(&mut self, path: &str) {
        let package_json_content = fs::read_to_string(&path).expect("Failed to read file");

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
    }
}
