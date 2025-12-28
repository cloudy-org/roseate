use std::{collections::HashSet, hash::{Hash, Hasher}};

pub type ImageModifications = HashSet<ImageModification>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageModification {
    Resize(u32, u32),
}

impl Hash for ImageModification {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // all Resize variants hash the same value
        std::mem::discriminant(self).hash(state);
    }
}