use rstest::{fixture, rstest};
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Write;
use tar::Builder;
use tar::Header;
use tempfile::TempDir;
use tempfile::tempdir;


// pacman database version (lib/libalpm/be_local.c)
const ALPM_DB_VERSION: &str = "9";

#[derive(Hash, Eq, PartialEq, Debug)]
struct Package {
    name: String,
    base: String,
    version: String,
    depends: Vec<String>,
    makedepends: Vec<String>,
}

impl Package {
    fn new(
        name: &str,
        base: &str,
        version: &str,
        depends: Vec<String>,
        makedepends: Vec<String>,
    ) -> Package {
        Package {
            name: name.to_string(),
            base: base.to_string(),
            version: version.to_string(),
            depends: depends,
            makedepends: makedepends,
        }
    }

    fn desc(&self) -> String {
        let mut desc = String::from("");

        let name = format!("%NAME%\n{}\n", self.name);
        desc.push_str(&name);

        let base = format!("%BASE%\n{}\n", self.base);
        desc.push_str(&base);

        if !self.depends.is_empty() {
            desc.push_str("%DEPENDS%\n");
            for dep in self.depends.iter() {
                desc.push_str(dep);
                desc.push_str("\n");
            }
            desc.push_str("\n");
        }

        if !self.makedepends.is_empty() {
            desc.push_str("%MAKEDEPENDS%\n");
            for dep in self.makedepends.iter() {
                desc.push_str(dep);
                desc.push_str("\n");
            }
            desc.push_str("\n");
        }

        desc
    }

    fn path(&self) -> String {
        format!("{}-{}/desc", self.name, self.version)
    }

    fn tarheader(&self) -> Header {
        let mut header = Header::new_gnu();
        let desc = self.desc();
        let datalen = u64::try_from(desc.len()).unwrap();
        header.set_path(self.path()).unwrap();
        header.set_size(datalen);
        header.set_mode(0o644);
        header.set_cksum();

        header
    }
}


fn init_repodb(reponame: String, packages: Vec<Package>) -> (TempDir, String) {
    let tempdir = tempdir().unwrap();
    let dbpath = tempdir.path().display().to_string();

    // local dir
    let localdir = tempdir.path().join("local");
    fs::create_dir(&localdir).unwrap();

    let mut file = File::create(localdir.join("ALPM_DB_VERSION")).unwrap();
    file.write_all(ALPM_DB_VERSION.as_bytes()).unwrap();

    // sync dir
    let syncdir = tempdir.path().join("sync");
    fs::create_dir(&syncdir).unwrap();

    let dbloc = syncdir.join(format!("{}.db", reponame));
    create_db(dbloc.display().to_string(), packages);

    (tempdir, dbpath)
}


fn create_db(dbloc: String, pkgs: Vec<Package>) {
    let mut archive = Builder::new(Vec::new());

    for pkg in pkgs {
        let header = pkg.tarheader();
        let desc = pkg.desc();
        let data = desc.as_bytes();
        archive.append(&header, data).unwrap();
    }

    archive.finish().unwrap();
    let data = archive.into_inner().unwrap();

    let mut afile = File::create(dbloc).unwrap();
    afile.write_all(&data).unwrap();
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
fn no_reverse_deps() -> (Vec<String>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new("testpkg1", "testpkg1", "1.0-1", vec![], vec![]);
    let reponame = "test";
    let repos = vec![reponame.to_string()];
    let pkgnames = vec![testpkg.name.clone()];
    let packages = vec![testpkg];
    let (rootdir, dbpath) = init_repodb(reponame.to_string(), packages);

    (pkgnames, Some(dbpath), repos, rootdir)
}

#[fixture]
fn reverse_deps() -> (Vec<String>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new("testpkg1", "testpkg1", "1.0-1", vec![], vec![]);
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1.0-1",
        vec![testpkg.name.clone()],
        vec![],
    );
    let pkgnames = vec![testpkg.name.clone(), testpkg2.name.clone()];
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages);
    let repos = vec![reponame.to_string()];

    (pkgnames, Some(dbpath), repos, tempdir)
}

#[fixture]
fn multiple_deps() -> (Vec<String>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new("testpkg1", "testpkg1", "1.0-1", vec![], vec![]);
    let testpkg2 = Package::new("testpkg2", "testpkg2", "1.0-1", vec![], vec![testpkg.name.clone()]);
    let testpkg3 = Package::new("testpkg3", "testpkg3", "1-1", vec![testpkg.name.clone(), testpkg2.name.clone()], vec![]);
    let pkgnames = vec![testpkg.name.clone(), testpkg2.name.clone(), testpkg3.name.clone()];
    let packages = vec![testpkg3, testpkg2, testpkg];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages);
    let repos = vec![reponame.to_string()];

    (pkgnames, Some(dbpath), repos, tempdir)
}

#[fixture]
fn reverse_make_deps() -> (Vec<String>, Option<String>, Vec<String>, TempDir) {
    let testpkg = Package::new("testpkg1", "testpkg1", "1.0-1", vec![], vec![]);
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1.0-1",
        vec![],
        vec![testpkg.name.clone()],
    );
    let pkgnames = vec![testpkg.name.clone(), testpkg2.name.clone()];
    let packages = vec![testpkg, testpkg2];

    let reponame = "test";
    let (tempdir, dbpath) = init_repodb(reponame.to_string(), packages);
    let repos = vec![reponame.to_string()];

    (pkgnames, Some(dbpath), repos, tempdir)
}

#[rstest]
fn test_no_reverse_deps(no_reverse_deps: (Vec<String>, Option<String>, Vec<String>, TempDir)) {
    let pkgnames = no_reverse_deps.0;
    let dbpath = no_reverse_deps.1;
    let repos = no_reverse_deps.2;

    let res = rebuilder::run(pkgnames.clone(), dbpath, repos, None).unwrap();
    assert_eq!(pkgnames[0], res.trim());
}

#[rstest]
fn test_reverse_deps(reverse_deps: (Vec<String>, Option<String>, Vec<String>, TempDir)) {
    let pkgnames = reverse_deps.0.clone();
    let pkgname = &pkgnames[0];
    let dbpath = reverse_deps.1;
    let repos = reverse_deps.2;

    let res = rebuilder::run(vec![pkgname.to_string()], dbpath, repos, None).unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(pkgnames, res_pkgs);
}

#[rstest]
fn test_reverse_make_deps(reverse_make_deps: (Vec<String>, Option<String>, Vec<String>, TempDir)) {
    let pkgnames = reverse_make_deps.0.clone();
    let pkgname = &pkgnames[0];
    let dbpath = reverse_make_deps.1;
    let repos = reverse_make_deps.2;

    let res = rebuilder::run(vec![pkgname.to_string()], dbpath, repos, None).unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(pkgnames, res_pkgs);
}

#[rstest]
fn test_multiple_makedeps(multiple_deps: (Vec<String>, Option<String>, Vec<String>, TempDir)) {
    let pkgnames = multiple_deps.0.clone();
    let pkgname = &pkgnames[0];
    let dbpath = multiple_deps.1;
    let repos = multiple_deps.2;

    let res = rebuilder::run(vec![pkgname.to_string()], dbpath, repos, None).unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(pkgnames, res_pkgs);
}
