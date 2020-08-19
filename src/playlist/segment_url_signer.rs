use crate::playlist::PlaylistRewriter;
use url::Url;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_signing() {
        let signer = UrlSigner::new("foobar".to_string());
        let url = Url::parse("https://example.com/23.ts").unwrap();
        let url = signer.sign(url, 23);

        assert_eq!(
            "https://example.com/23.ts?e=23&h=e5030a591d2dd923f90d29600b0c02e458c0bc344b1ad8eb71a26cf636988b62",
            url.into_string()
        );
    }
}

struct UrlSigner {
    key: String,
}

impl UrlSigner {
    fn new(key: String) -> UrlSigner {
        UrlSigner {
            key
        }
    }

    fn sign(&self, mut url: Url, expiry_timestamp: u64) -> Url {
        let mut hmac = self.new_hmac();

        let mut content_to_sign = String::from(url.path());
        content_to_sign.push_str(&expiry_timestamp.to_string());

        hmac.update(&content_to_sign.as_bytes());
        let signature = hmac.finalize();
        let signature = hex::encode(signature.into_bytes());

        url.query_pairs_mut()
            .append_pair("e", &expiry_timestamp.to_string())
            .append_pair("h", &signature);

        url
    }

    fn new_hmac(&self) -> Hmac<Sha256> {
        Hmac::<Sha256>::new_varkey(self.key.as_bytes())
            .expect("HMAC can take key of any size")
    }
}
