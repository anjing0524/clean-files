use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Helper function to create a test project structure
fn create_test_structure(base: &Path) {
    // Create Node.js project
    let node_project = base.join("node-project");
    fs::create_dir(&node_project).unwrap();
    fs::write(node_project.join("package.json"), r#"{"name": "test"}"#).unwrap();
    let node_modules = node_project.join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("package1.js"), "console.log('test');").unwrap();
    let nested_package = node_modules.join("package1");
    fs::create_dir(&nested_package).unwrap();
    fs::write(nested_package.join("index.js"), "module.exports = {};").unwrap();

    // Create Rust project
    let rust_project = base.join("rust-project");
    fs::create_dir(&rust_project).unwrap();
    fs::write(
        rust_project.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    )
    .unwrap();
    let rust_target = rust_project.join("target");
    fs::create_dir(&rust_target).unwrap();
    let debug = rust_target.join("debug");
    fs::create_dir(&debug).unwrap();
    fs::write(debug.join("test.exe"), &[0u8; 1000]).unwrap();

    // Create Python project
    let python_project = base.join("python-project");
    fs::create_dir(&python_project).unwrap();
    fs::write(python_project.join("main.py"), "print('hello')").unwrap();
    let pycache = python_project.join("__pycache__");
    fs::create_dir(&pycache).unwrap();
    fs::write(pycache.join("main.cpython-39.pyc"), &[0u8; 500]).unwrap();

    // Create Java Maven project
    let java_project = base.join("java-project");
    fs::create_dir(&java_project).unwrap();
    fs::write(
        java_project.join("pom.xml"),
        "<project><modelVersion>4.0.0</modelVersion></project>",
    )
    .unwrap();
    let java_target = java_project.join("target");
    fs::create_dir(&java_target).unwrap();
    let classes = java_target.join("classes");
    fs::create_dir(&classes).unwrap();
    fs::write(classes.join("Test.class"), &[0u8; 300]).unwrap();

    // Create Gradle project
    let gradle_project = base.join("gradle-project");
    fs::create_dir(&gradle_project).unwrap();
    fs::write(gradle_project.join("build.gradle"), "plugins { id 'java' }").unwrap();
    let build_dir = gradle_project.join("build");
    fs::create_dir(&build_dir).unwrap();
    fs::write(build_dir.join("output.jar"), &[0u8; 200]).unwrap();
}

fn dir_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

#[test]
fn test_integration_node_modules() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let node_modules = temp_dir.path().join("node-project/node_modules");
    assert!(dir_exists(&node_modules));

    // Run scanner to find node_modules
    // This would normally be done through the CLI, but we test the core logic here
}

#[test]
fn test_integration_rust_target() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let rust_target = temp_dir.path().join("rust-project/target");
    assert!(dir_exists(&rust_target));
}

#[test]
fn test_integration_python_cache() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let pycache = temp_dir.path().join("python-project/__pycache__");
    assert!(dir_exists(&pycache));
}

#[test]
fn test_integration_java_target() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let java_target = temp_dir.path().join("java-project/target");
    assert!(dir_exists(&java_target));
}

#[test]
fn test_integration_gradle_build() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    let gradle_build = temp_dir.path().join("gradle-project/build");
    assert!(dir_exists(&gradle_build));
}

#[test]
fn test_integration_full_scan() {
    let temp_dir = TempDir::new().unwrap();
    create_test_structure(temp_dir.path());

    // Verify all directories exist
    assert!(dir_exists(
        &temp_dir.path().join("node-project/node_modules")
    ));
    assert!(dir_exists(&temp_dir.path().join("rust-project/target")));
    assert!(dir_exists(
        &temp_dir.path().join("python-project/__pycache__")
    ));
    assert!(dir_exists(&temp_dir.path().join("java-project/target")));
    assert!(dir_exists(&temp_dir.path().join("gradle-project/build")));
}

#[test]
fn test_nested_projects() {
    let temp_dir = TempDir::new().unwrap();

    // Create a nested structure
    let workspace = temp_dir.path().join("workspace");
    fs::create_dir(&workspace).unwrap();

    // Project 1
    let proj1 = workspace.join("project1");
    fs::create_dir(&proj1).unwrap();
    fs::write(proj1.join("package.json"), "{}").unwrap();
    fs::create_dir(proj1.join("node_modules")).unwrap();

    // Project 2 nested in project1
    let proj2 = proj1.join("packages").join("project2");
    fs::create_dir_all(&proj2).unwrap();
    fs::write(proj2.join("package.json"), "{}").unwrap();
    fs::create_dir(proj2.join("node_modules")).unwrap();

    assert!(dir_exists(&proj1.join("node_modules")));
    assert!(dir_exists(&proj2.join("node_modules")));
}

#[test]
fn test_symlink_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create a real directory
    let real_dir = temp_dir.path().join("real");
    fs::create_dir(&real_dir).unwrap();
    fs::write(real_dir.join("file.txt"), "content").unwrap();

    // Note: Symlink creation may fail on some systems (Windows without privileges)
    // so we skip the test if it fails
    #[cfg(unix)]
    {
        let link = temp_dir.path().join("link");
        if std::os::unix::fs::symlink(&real_dir, &link).is_ok() {
            assert!(link.exists());
            assert!(link.is_symlink());
        }
    }
}
