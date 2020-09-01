use reqwest::{Client, Method, RequestBuilder};
use url::Url;

pub struct HttpClient {
    inner: Client,
    base_url: Url,
}

impl HttpClient {
    pub fn new(inner: Client, base_url: Url) -> Self {
        HttpClient { inner, base_url }
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.inner.get(self.url(url))
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        self.inner.post(self.url(url))
    }

    pub fn put(&self, url: &str) -> RequestBuilder {
        self.inner.put(self.url(url))
    }

    pub fn patch(&self, url: &str) -> RequestBuilder {
        self.inner.patch(self.url(url))
    }

    pub fn delete(&self, url: &str) -> RequestBuilder {
        self.inner.delete(self.url(url))
    }

    pub fn head(&self, url: &str) -> RequestBuilder {
        self.inner.head(self.url(url))
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.inner.request(method, self.url(url))
    }

    fn url(&self, url: &str) -> Url {
        self.base_url.join(url).unwrap()
    }
}
