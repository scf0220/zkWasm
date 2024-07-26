use chksum_md5 as md5;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;

pub(crate) static ZKSYNC_KEY_URL: Lazy<HashMap<u32, ZKSyncKey>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        22,
        ZKSyncKey {
            k: 22,
            monomial_key_url: String::from(
                "https://storage.googleapis.com/universal-setup/setup_2%5E22.key",
            ),
            monomial_key_md5: String::from("4a337eff33368320b579a0319485170f"),
            lagrange_key_url: String::from(
                "https://storage.googleapis.com/universal-setup/setup_2%5E22_lagrange.key",
            ),
            lagrange_key_md5: String::from("c651928e884b8f1b02f970bba0287013"),
        },
    );
    map.insert(
        23,
        ZKSyncKey {
            k: 23,
            monomial_key_url: String::from(
                "https://storage.googleapis.com/universal-setup/setup_2%5E23.key",
            ),
            monomial_key_md5: String::from("e47ade70fa43b1448f726e3e41beb1b8"),
            lagrange_key_url: String::from(
                "https://storage.googleapis.com/universal-setup/setup_2%5E23_lagrange.key",
            ),
            lagrange_key_md5: String::from("8f9442445de62f4bc37f86d8c15ba390"),
        },
    );
    map
});

#[derive(Debug, Clone)]
pub struct ZKSyncKey {
    pub k: i32,
    pub monomial_key_url: String,
    pub monomial_key_md5: String,
    pub lagrange_key_url: String,
    pub lagrange_key_md5: String,
}

impl ZKSyncKey {
    pub fn check_setup_key_file(&mut self) -> anyhow::Result<()> {
        self.check_monomial()?;
        self.check_lagrange()?;
        Ok(())
    }
    pub fn get_local_monomial_path(&mut self) -> String {
        format!("./setup_2^{:?}.key", self.k)
    }
    pub fn get_local_lagrange_path(&mut self) -> String {
        format!("./setup_2^{:?}_lagrange.key", self.k)
    }
    pub fn check_monomial(&mut self) -> anyhow::Result<()> {
        let local_path = self.get_local_monomial_path();
        let file = File::open(local_path.clone());
        if file.is_err() {
            download_setup_key(self.monomial_key_url.clone(), local_path.clone())?
        }
        let file = File::open(local_path.clone())?;
        let digest = md5::chksum(file)?;
        assert_eq!(digest.to_string(), self.monomial_key_md5);
        println!(
            "k={} type=monomial patch={:?} check succ",
            self.k, local_path
        );
        return Ok(());
    }

    pub fn check_lagrange(&mut self) -> anyhow::Result<()> {
        let local_path = self.get_local_lagrange_path();
        let file = File::open(local_path.clone());
        if file.is_err() {
            download_setup_key(self.lagrange_key_url.clone(), local_path.clone())?
        }
        let file = File::open(local_path.clone())?;
        let digest = md5::chksum(file)?;
        assert_eq!(digest.to_string(), self.lagrange_key_md5);
        println!(
            "k={} type=lagrange path={:?} check succ",
            self.k, local_path
        );
        return Ok(());
    }
}

pub fn download_setup_key(url: String, local_path: String) -> anyhow::Result<()> {
    println!(
        "begin download key url={:?} local_path={:?}",
        url, local_path
    );

    if Path::new(&local_path).exists() {
        fs::remove_file(local_path.clone())?;
    }
    let mut reader = reqwest::blocking::get(&url)?;
    let mut file = File::create(local_path)?;
    copy(&mut reader, &mut file)?;
    return Ok(());
}
