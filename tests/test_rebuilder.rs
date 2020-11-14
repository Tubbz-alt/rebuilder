use rstest::{fixture, rstest};
use std::fs;
use tempfile::TempDir;
use tempfile::Builder;
use std::fs::File;
use std::io::{Write};
//use flate2::Compression;
//use flate2::write::GzEncoder;
use std::collections::HashMap;
use tar::Builder as TarBuilder;


fn get_pkg_desc(pkgname: String, depends: Vec<String>, makedepends: Vec<String>) -> String {
    let fields: HashMap<&str, &str> = [("FILENAME", "foo.tar.zst"),
                                       ("NAME", &pkgname),
                                       ("BASE", &pkgname), 
                                       ("VERSION", "1.0-1"),
                                       ("DESC", "pkg"),
                                       ("ARCH", "x86_64"),
                                       ("BUILDDATE", "1590150671"),
                                       ("PACKAGER", "Crab"),
                                       ("URL", "https://archlinux.org"),
                                       ("LICENSES", "MIT"),
        ].iter().cloned().collect();

    let mut desc = String::from("");

    // Common fields
    for (key, val) in fields.iter() {
        let key = format!("%{}%\n", key.to_owned());
        desc.push_str(&key);
        let val = format!("{}\n", val);
        desc.push_str(&val);
    }

    if !depends.is_empty() {
        desc.push_str("%DEPENDS%\n");
        for dep in depends.iter() {
            desc.push_str(dep);
        }
        desc.push_str("\n");
    }

    if !makedepends.is_empty() {
        desc.push_str("%MAKEDEPENDS%\n");
        for dep in makedepends.iter() {
            desc.push_str(dep);
        }
        desc.push_str("\n");
    }

    dbg!(desc.clone());

    desc
}

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
fn no_reverse_deps() -> (Vec<String>, Option<String>, TempDir) {
    let rootdir = Builder::new().prefix("example").tempdir().unwrap();
    let dbpath = rootdir.path().display().to_string();
    let pkgname = String::from("testpkg1");
    let repos = vec!["core", "extra", "community", "multilib"];
    let pkgnames = vec![pkgname];

    // /var/lib/pacman/local
    let localdir = format!("{}/local", dbpath);
    fs::create_dir(localdir).unwrap();
    let file_path = format!("{}/local/ALPM_DB_VERSION", dbpath);
    let mut file = File::create(file_path).unwrap();
    // TODO: define const
    writeln!(file, "9").unwrap();

    // /var/lib/pacman/sync
    let syncdir = format!("{}/sync", dbpath);
    fs::create_dir(syncdir).unwrap();

    // every db needs at least one pkg to be valid.
    for repo in &repos {
        dbg!("loop 1");
        let name = format!("{}.db", repo);
        let repodir = Builder::new().prefix(&name).tempdir().unwrap();
        let repodirstr = repodir.path().display().to_string();
        let placeholder = String::from("placeholder");
        let placeholderstr = placeholder.clone();
        let desc = get_pkg_desc(placeholder, vec![], vec![]);

        fs::create_dir(format!("{}/{}", repodirstr, placeholderstr)).unwrap();
        let file_path = format!("{}/{}/desc", repodirstr, placeholderstr);
        let desc_path = file_path.clone();
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "{}", desc).unwrap();

        let db = File::create("repo.tar").unwrap();
        let mut tar = TarBuilder::new(file);
        dbg!("woop");
        //(desc_path, format!("{}-1.0-1/desc", placeholderstr)).unwrap();
        //tar.append_path_with_name(desc_path, format!("{}-1.0-1/desc", placeholderstr)).unwrap();
        dbg!("ninin");
    }
    /*
    for pkg in &pkgnames {
        let path = syncdir.clone().join(format!("{}-1-1", pkg));
        dbg!(path.as_path().display());
        fs::create_dir_all(path).unwrap();
    }
    */

    (pkgnames, Some(dbpath), rootdir)
}

#[rstest]
fn test_reverse_deps(no_reverse_deps: (Vec<String>, Option<String>, TempDir)) {
    println!("lala");
    let pkgnames = no_reverse_deps.0;
    let dbpath = no_reverse_deps.1;
    let dbpath2 = dbpath.clone();
    println!("googgo");
    for entry in fs::read_dir(dbpath.unwrap()).unwrap() {
        dbg!(entry);
    }

    //rebuilder::run(pkgnames, dbpath, None).unwrap();
}
