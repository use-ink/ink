

/// Type state for an empty type list.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TupleVecSentinel;

/// Type state to communicate the end of a type list.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TupleVecEnd;

/// Type node containing the actual type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TupleVecLeaf<T>{
    item: T,
}

impl<T> From<T> for TupleVecLeaf<T> {
    fn from(item: T) -> Self {
        Self { item }
    }
}

impl Default for TupleVecLeaf<TupleVecSentinel> {
    fn default() -> Self {
        Self { item: TupleVecSentinel }
    }
}

/// Used as a constrain for type list tails.
pub trait TupleVec {
    const LEN: usize;

    #[inline(always)]
    fn len(&self) -> usize {
        Self::LEN
    }
}

/// A stack-like data structure that can contain objects with different types.
#[derive(Debug, PartialEq, Eq)]
pub struct TupleVecNode<T, Rest> {
    head: TupleVecLeaf<T>,
    rest: Rest,
}

impl TupleVecNode<TupleVecSentinel, TupleVecSentinel> {
    /// Creates a new empty tuple vector.
    pub fn new() -> Self {
        Self { head: Default::default(), rest: TupleVecSentinel }
    }
}

impl TupleVec for TupleVecNode<TupleVecSentinel, TupleVecSentinel> {
    const LEN: usize = 0;
}

impl<Head, Rest> TupleVec for TupleVecNode<Head, Rest>
where
    Rest: TupleVec,
{
    const LEN: usize = 1 + <Rest as TupleVec>::LEN;
}

impl TupleVec for TupleVecEnd {
    const LEN: usize = 0;
}

impl TupleVecNode<TupleVecSentinel, TupleVecSentinel> {
    /// Pushes an element to the empty tuple vector.
    pub fn push<T>(self, item: T) -> TupleVecNode<T, TupleVecEnd> {
        TupleVecNode { head: TupleVecLeaf::from(item), rest: TupleVecEnd }
    }
}

impl<Head, Rest> TupleVecNode<Head, Rest>
where
    Rest: TupleVec,
{
    /// Pushes another element to the tuple vector.
    pub fn push<T>(self, item: T) -> TupleVecNode<T, Self> {
        TupleVecNode { head: TupleVecLeaf::from(item), rest: self, }
    }

    /// Pops the head element from the tuple vector.
    pub fn pop(self) -> (Head, Rest) {
        (self.head.item, self.rest)
    }

    /// Returns a reference to the top element and provide a reference
    /// to the rest for further iteration.
    pub fn next(&self) -> (&Head, &Rest) {
        (&self.head.item, &self.rest)
    }
}

impl<Head, Rest> serde::Serialize for TupleVecNode<Head, Rest>
where
    Head: serde::Serialize,
    Rest: TupleVec + SerializeElement,
{
    /// Serializes this tuple vector as a sequence of its values.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq as _;
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        self.serialize_element(&mut seq)?;
        seq.end()
    }
}

pub trait SerializeElement {
    fn serialize_element<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeSeq;
}

impl SerializeElement for TupleVecEnd {
    fn serialize_element<S>(&self, _serializer: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeSeq,
    {
        Ok(())
    }
}

impl<Head, Rest> SerializeElement for TupleVecNode<Head, Rest>
where
    Head: serde::Serialize,
    Rest: TupleVec + SerializeElement,
{
    fn serialize_element<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: serde::ser::SerializeSeq,
    {
        serializer.serialize_element(&self.head.item)?;
        self.rest.serialize_element(serializer)
    }
}

/// Creates a new tuple vector filled with the given values.
macro_rules! tuple_vec {
    [ $($elem:expr),* ] => {
        {
            let vec = crate::TupleVecNode::new();
            $(
                let vec = vec.push($elem);
            )*
            let result = crate::TupleVecNode::new();
            // Now reinsert to reverse the reversed order from before.
            $(
                #[allow(unused)]
                let (head, vec) = vec.pop();
                let result = result.push(head);
                let _ = $elem;
            )*
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_macro() {
        let v = tuple_vec!(
            1,
            true,
            "Hello, World!",
            1.337,
            'a'
        );
        println!("{:#?}", v);

        assert_eq!(v.len(), 5);

        let (&head, rest) = v.next();
        assert_eq!(head, 1);
        let (&head, rest) = rest.next();
        assert_eq!(head, true);
        let (&head, rest) = rest.next();
        assert_eq!(head, "Hello, World!");
        let (&head, rest) = rest.next();
        assert_eq!(head, 1.337);
        let (&head, _rest) = rest.next();
        assert_eq!(head, 'a');

        assert_eq!(json::to_string(&v).unwrap(), "[1,true,\"Hello, World!\",1.337,\"a\"]");
    }
}
