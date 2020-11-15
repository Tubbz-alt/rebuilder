use rstest::{fixture, rstest};
use std::fs;
use tempfile::TempDir;
use tempfile::Builder;
use std::fs::File;
use std::io::{Write};
use std::collections::HashMap;
use tar::Builder as TarBuilder;
use tar::Header;


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
        desc.push_str("\n");
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

    desc
}

#[fixture]
pub fn invalid_data() -> (Vec<String>, Option<String>) {
    let pkgnames = vec![String::from("testpkg1")];
    let dbpath = Some(String::from("/non-existant-path")); 

    (pkgnames, dbpath)
}

#[rstest]
#[should_panic]
fn should_panic(invalid_data: (Vec<String>, Option<String>)) {
    let pkgnames = invalid_data.0;
    let dbpath = invalid_data.1;
    rebuilder::run(pkgnames, dbpath, vec![], None).unwrap();
}


#[fixture]
fn no_reverse_deps() -> (Vec<String>, Option<String>, TempDir) {
    let rootdir = Builder::new().prefix("example").tempdir().unwrap();
    let dbpath = rootdir.path().display().to_string();
    let pkgname = String::from("testpkg1");
    let reponame = String::from("test");
    let _repos = vec![reponame.clone()];
    let pkgnames = vec![pkgname.clone()];

    // /var/lib/pacman/local
    let localdir = format!("{}/local", dbpath);
    fs::create_dir(localdir).unwrap();
    let file_path = format!("{}/local/ALPM_DB_VERSION", dbpath);
    let mut file = File::create(file_path).unwrap();
    // TODO: define const
    writeln!(file, "9").unwrap();

    // /var/lib/pacman/sync
    let syncdir = format!("{}/sync", dbpath);
    fs::create_dir(&syncdir).unwrap();

    let desc = get_pkg_desc(pkgname, vec![], vec![]);
    let mut header = Header::new_gnu();

    let data = desc.as_bytes();
    header.set_path("testpkg1-1.0-1/desc").unwrap();
    header.set_size(188); // TODO: how to set size
    header.set_gid(0);
    header.set_uid(0);
    header.set_mode(0o644);
    header.set_cksum();

    let mut archive = TarBuilder::new(Vec::new());
    archive.append(&header, data).unwrap();
    archive.finish().unwrap();
    let data = archive.into_inner().unwrap();

    let mut afile = File::create(format!("{}/{}", syncdir, "test.db")).unwrap();
    afile.write_all(&data).unwrap();

    (pkgnames, Some(dbpath), rootdir)
}

#[rstest]
fn test_reverse_deps(no_reverse_deps: (Vec<String>, Option<String>, TempDir)) {
    let reponame = String::from("test");
    let pkgnames = no_reverse_deps.0;
    let dbpath = no_reverse_deps.1;

    let res = rebuilder::run(pkgnames.clone(), dbpath, vec![reponame], None).unwrap();
    assert_eq!(pkgnames[0], res.trim());
}
