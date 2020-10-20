use rstest::{fixture, rstest};
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use tempfile::tempdir;
use std::fs::File;
use std::io::{self, Write};



#[fixture]
pub fn invalid_data() -> (Vec<String>, Option<String>) {
    let pkgnames = vec![String::from("testpkg1")];
    let dbpath = Some(String::from("/non-existant-path")); 

    (pkgnames, dbpath)
}

//#[rstest]
//#[should_panic]
fn should_panic(invalid_data: (Vec<String>, Option<String>)) {
    let pkgnames = invalid_data.0;
    let dbpath = invalid_data.1;
    rebuilder::run(pkgnames, dbpath, None).unwrap();
}


#[fixture]
fn no_reverse_deps() -> (Vec<String>, Option<String>) {
    let pkgname = String::from("testpkg1");
    let pkgnames = vec![pkgname];
    let rootdir = Path::new("/tmp/.rebuilderd");

    fs::create_dir(rootdir).unwrap();

    let localdir = rootdir.join("local/");
    fs::create_dir_all(localdir).unwrap();

    // Create core,extra,community,multilib db's
    let syncdir = rootdir.join("sync");
    for pkg in &pkgnames {
        let path = syncdir.clone().join(format!("{}-1-1", pkg));
        dbg!(path.as_path().display());
        fs::create_dir_all(path).unwrap();
    }

    let file_path = syncdir.join("ALPM_DB_VERSION");
    let mut file = File::create(file_path).unwrap();
    // TODO: define const
    writeln!(file, "9").unwrap();

    let dbpath = Some(rootdir.display().to_string());

    (pkgnames, dbpath)
}


#[rstest]
fn test_reverse_deps(no_reverse_deps: (Vec<String>, Option<String>)) {
    let pkgnames = no_reverse_deps.0;
    let dbpath = no_reverse_deps.1;
    let dbpath2 = dbpath.clone();

    /*
    for entry in fs::read_dir(dbpath.unwrap()).unwrap() {
        dbg!(entry);
    }
    */
    rebuilder::run(pkgnames, dbpath, None).unwrap();

    fs::remove_dir_all(dbpath2.unwrap());
}
