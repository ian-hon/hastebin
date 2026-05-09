use crate::{models::Paste, utils};

impl Paste {
    pub fn construct_complete_checksum(&self, checksum_passphrase: Option<String>) -> String {
        let mut partial = self.construct_partial_checksum();
        partial.push_str(
            &checksum_passphrase
                .unwrap_or_else(|| self.checksum_passphrase.clone().unwrap_or("".to_string())),
        );

        str::from_utf8(&utils::construct_digest(partial))
            .unwrap()
            .to_string()
    }

    pub fn construct_partial_checksum(&self) -> String {
        let mut p = self.clone();
        p.checksum_passphrase = None;
        p.views = 0;

        str::from_utf8(&utils::construct_digest(serde_json::to_string(&p).unwrap()))
            .unwrap()
            .to_string()
    }

    pub fn validate_checksum(&self, checksum_passphrase: String) -> bool {
        self.construct_complete_checksum(Some(checksum_passphrase))
            .eq(&self.construct_complete_checksum(None))
    }
}
