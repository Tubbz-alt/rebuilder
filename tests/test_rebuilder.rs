use rstest::{fixture, rstest};
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Write;
use tar::Builder as TarBuilder;
use tar::Header;
use tempfile::Builder;
use tempfile::TempDir;

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
            }
            desc.push_str("\n");
        }

        if !self.makedepends.is_empty() {
            desc.push_str("%MAKEDEPENDS%\n");
            for dep in self.makedepends.iter() {
                desc.push_str(dep);
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


fn init_repo() -> (TempDir, String, String) {
    // TODO: do we need a root dir
    let rootdir = Builder::new().prefix("no_reverse_deps").tempdir().unwrap();
    let dbpath = rootdir.path().display().to_string();
    // local dir
    let localdir = format!("{}/local", dbpath);
    fs::create_dir(localdir).unwrap();
    let file_path = format!("{}/local/ALPM_DB_VERSION", dbpath);
    let mut file = File::create(file_path).unwrap();
    writeln!(file, "{}", ALPM_DB_VERSION).unwrap();

    // sync dir
    let syncdir = format!("{}/sync", dbpath);
    fs::create_dir(&syncdir).unwrap();

    (rootdir, dbpath, syncdir)
}


fn create_db(dbloc: String, pkgs: Vec<Package>) {
    let mut archive = TarBuilder::new(Vec::new());

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
    let reponame = String::from("test");
    let (rootdir, dbpath, syncdir) = init_repo();

    let repos = vec![reponame.clone()];
    let testpkg = Package::new("testpkg1", "testpkg1", "1.0-1", vec![], vec![]);
    let pkgnames = vec![testpkg.name.clone()];

    let dbloc = format!("{}/{}.db", syncdir, reponame);
    create_db(dbloc, vec![testpkg]);

    (pkgnames, Some(dbpath), repos, rootdir)
}

#[fixture]
fn reverse_deps() -> (Vec<String>, Option<String>, Vec<String>, TempDir) {
    let reponame = String::from("test");
    let (rootdir, dbpath, syncdir) = init_repo();

    let repos = vec![reponame.clone()];
    let testpkg = Package::new("testpkg1", "testpkg1", "1.0-1", vec![], vec![]);
    let testpkg2 = Package::new(
        "testpkg2",
        "testpkg2",
        "1.0-1",
        vec!["testpkg1".to_string()],
        vec![],
    );

    let pkgnames = vec![testpkg.name.clone(), testpkg2.name.clone()];
    let dbloc = format!("{}/{}.db", syncdir, reponame);
    create_db(dbloc, vec![testpkg, testpkg2]);

    (pkgnames, Some(dbpath), repos, rootdir)
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
