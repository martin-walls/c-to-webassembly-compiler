/// A type representing an identifier in the IR.
/// E.g. variable identifiers, function identifiers.
///
/// The trait is an abstraction for generating new identifiers.
pub trait Id {
    /// Generate the initial id, when no IDs exist yet. (Id 0)
    fn initial_id() -> Self;
    /// Generate a new id, given the current max id. (Id n+1)
    fn next_id(&self) -> Self;
    /// Get a u64 representation of the id
    fn as_u64(&self) -> u64;
}

#[derive(Debug)]
pub struct IdGenerator<T: Id + Clone> {
    max_id: Option<T>,
}

impl<T: Id + Clone> IdGenerator<T> {
    pub fn new() -> Self {
        IdGenerator { max_id: None }
    }

    pub fn new_id(&mut self) -> T {
        let new_id = match &self.max_id {
            None => T::initial_id(),
            Some(id) => id.next_id(),
        };
        self.max_id = Some(new_id.to_owned());
        new_id
    }
}
