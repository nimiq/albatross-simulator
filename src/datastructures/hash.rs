use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct Hasher(Sha256);

impl Hasher {
    pub fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.0.input(data)
    }

    pub fn chain<B: AsRef<[u8]>>(mut self, data: B) -> Self where Self: Sized {
        self.input(data);
        self
    }

    pub fn result(self) -> Hash {
        Hash::from(self.0.result().as_slice())
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn hash<T: AsRef<[u8]>>(data: T) -> Self {
        Hasher::default().chain(data).result()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl<'a> From<&'a [u8]> for Hash {
    fn from(slice: &'a [u8]) -> Self {
        assert_eq!(slice.len(), 32, "Tried to create instance with slice of wrong length");
        let mut a = [0u8; 32];
        a.clone_from_slice(&slice[0..32]);
        Hash(a)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> From<&'a Hash> for Vec<u8> {
    fn from(hash: &'a Hash) -> Self {
        hash.to_vec()
    }
}

impl From<Hash> for Vec<u8> {
    fn from(hash: Hash) -> Self {
        hash.to_vec()
    }
}
