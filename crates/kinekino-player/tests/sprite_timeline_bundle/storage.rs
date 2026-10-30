use super::custody::Guard;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Write, copy},
    path::{Path, PathBuf},
};
pub struct Bundle {
    pub root: PathBuf,
    pub assets: Vec<Value>,
    pub receipts: Vec<Value>,
    ids: BTreeSet<String>,
    guards: Vec<Guard>,
    objects: BTreeSet<String>,
}
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn write_new(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}
impl Bundle {
    pub fn new(root: PathBuf) -> Self {
        fs::create_dir(&root).unwrap();
        for name in ["packages", "objects", "decisions"] {
            fs::create_dir(root.join(name)).unwrap();
        }
        Self {
            root,
            assets: Vec::new(),
            receipts: Vec::new(),
            ids: BTreeSet::new(),
            guards: Vec::new(),
            objects: BTreeSet::new(),
        }
    }
    pub fn import(&mut self, id: &str, path: &Path, expected: &str, mime: &str) -> Value {
        let original = Guard::acquire(path);
        assert_eq!(original.sha256, expected, "input {id}");
        let target = self.root.join("objects").join(expected);
        if self.objects.insert(expected.into()) {
            let mut from = File::open(path).unwrap();
            let mut to = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .unwrap();
            copy(&mut from, &mut to).unwrap();
            to.sync_all().unwrap();
            drop(to);
        }
        let copied = Guard::acquire(&target);
        assert_eq!(copied.sha256, expected);
        original.assert_unchanged();
        self.receipts.push(json!({"asset_id":id,"source_path":path,"source":original.evidence(),"copy_path":target,"copy":copied.evidence(),"content_equal":true,"physical_identity_claim":"separate_file; historical identity preserved"}));
        self.guards.push(original);
        self.guards.push(copied);
        self.register(id, expected, fs::metadata(target).unwrap().len(), mime)
    }
    pub fn generated(&mut self, id: &str, value: &Value, mime: &str) -> Value {
        let bytes = serde_json::to_vec(value).unwrap();
        let digest = sha(&bytes);
        let path = self.root.join("objects").join(&digest);
        if self.objects.insert(digest.clone()) {
            write_new(&path, &bytes);
        }
        let guard = Guard::acquire(&path);
        assert_eq!(guard.sha256, digest);
        self.receipts.push(json!({"asset_id":id,"generated_path":path,"generated":guard.evidence(),"kind":"explicit_normalization; not a new decoder run"}));
        self.guards.push(guard);
        self.register(id, &digest, bytes.len() as u64, mime)
    }
    fn register(&mut self, id: &str, digest: &str, bytes: u64, mime: &str) -> Value {
        assert!(self.ids.insert(id.into()), "duplicate asset ID {id}");
        self.assets.push(
            json!({"asset_id":id,"sha256":digest,"bytes":bytes.to_string(),"media_type":mime}),
        );
        json!({"asset_id":id,"sha256":digest})
    }
    pub fn bytes(&self, reference: &Value) -> Vec<u8> {
        let path = self
            .root
            .join("objects")
            .join(reference["sha256"].as_str().unwrap());
        assert!(fs::metadata(&path).unwrap().len() <= 65536);
        fs::read(path).unwrap()
    }
    pub fn json(&self, reference: &Value) -> Value {
        serde_json::from_slice(&self.bytes(reference)).unwrap()
    }
    pub fn text(&self, reference: &Value) -> String {
        String::from_utf8(self.bytes(reference)).unwrap()
    }
    pub fn assert_all_unchanged(&self) {
        for guard in &self.guards {
            guard.assert_unchanged();
        }
    }
    pub fn seal_file(&mut self, name: &str, value: &Value) -> String {
        let path = self.root.join(name);
        let bytes = serde_json::to_vec_pretty(value).unwrap();
        write_new(&path, &bytes);
        let guard = Guard::acquire(&path);
        let digest = guard.sha256.clone();
        self.guards.push(guard);
        digest
    }
}
