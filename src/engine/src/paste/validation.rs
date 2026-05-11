use crate::{models::Paste, utils};

impl Paste {
    pub fn construct_partial_checksum(&self) -> String {
        let mut p = self.clone();

        // remove both checksum and views to create hash
        p.checksum_passphrase = None;
        p.views = 0;

        utils::construct_digest(serde_json::to_string(&p).unwrap())
    }

    pub fn construct_checksum_pair(&self) -> Option<(String, String)> {
        let mut partial = self.construct_partial_checksum();
        partial.push_str(&self.checksum_passphrase.clone()?);

        Some((
            self.construct_partial_checksum(),
            utils::construct_digest(partial),
        ))
    }
}
