use rsa::Hash;
use std::collections::HashMap;

pub struct QueryParam {
    pub inner: HashMap<String, Vec<String>>,
}

impl QueryParam {
    pub fn new() -> Self {
        return Self {
            inner: HashMap::default(),
        };
    }
    pub fn adds(&mut self, k: String, v: Vec<String>) {
        if self.inner.contains_key(&k) {
            self.inner.get_mut(&k).unwrap().extend_from_slice(&v);
        } else {
            self.inner.insert(k, v);
        }
    }

    pub fn add(&mut self, k: String, v: String) {
        self.adds(k, vec![v]);
    }

    pub fn encode(&self) -> String {
        let mut keys = Vec::with_capacity(self.inner.len());
        for k in self.inner.keys() {
            keys.push(k.to_string());
        }
       // keys.sort();
        let mut buf = vec![];
        for key in keys {
            let key_escaped = urlencoding::encode(&key);
            for v in self.inner.get(&key).unwrap().iter() {
                buf.push(format!("{}={}", key_escaped, urlencoding::encode(v)));
            }
        }
        return buf.join("&");
    }
}
