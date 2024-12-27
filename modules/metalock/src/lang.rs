
use anchor_lang::prelude::Pubkey;

use crate::expr::*;

impl<I: Clone, Node: R<I>> UniversalAPI<I> for Node {}
trait UniversalAPI<I: Clone>: Sized + R<I> {
    fn equals<U: R<I>>(self, other: U) -> Equals<I, Self, U> {
        Equals(self, other, ph())
    }
}

impl<I: Clone, Node: R<Vec<I>>> VecAPI<I> for Node {}
trait VecAPI<I: Clone>: Sized + R<Vec<I>> {
    fn all<F: Fn(Ref<I>) -> Body, Body: R<bool>>(self, f: F) -> All<I, Self, Body> {
        All::new(self, f)
    }
    fn any<F: Fn(Ref<I>) -> Body, Body: R<bool>>(self, f: F) -> Any<I, Self, Body> {
        Any::new(self, f)
    }
    fn map<F: Fn(Ref<I>) -> Body, O: Clone, Body: R<O>>(self, f: F) -> Map<I, O, Self, Body> {
        Map::new(self, f)
    }
}

impl<L: HasLen + Clone, Node: R<L>> LenAPI<L> for Node {}
trait LenAPI<L: Clone + HasLen>: Sized + R<L> {
    fn len(self) -> Length<L, Self> {
        Length(self, ph())
    }
}

impl<Node: R<bool>> BoolAPI for Node {}
trait BoolAPI: Sized + R<bool> {
    fn not(self) -> Not<Self> {
        Not(self)
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        // All
        let comp = Val::new(vec![false, false, false]).all(|b| b.not());
        assert!(comp.eval() == true.into());
        let comp = Val::new(vec![false, false, true]).all(|b| b.not());
        assert!(comp.eval() == false.into());

        // Length
        let comp = Val::new(vec![false, false]).len();
        assert!(comp.eval() == 2u16.into());

        // Map
        let v = vec!["hi".to_string(), "there".to_string()];
        let comp = Val::new(v).map(|s| s.len());
        assert!(comp.eval() == vec![2u16, 5].into());

        // Any
        let key = Pubkey::new_unique();
        let v = vec![key.clone(), Pubkey::new_unique(), Pubkey::new_unique()];
        let comp = Val::new(v).any(|p| p.equals(Val::new(key)));
        println!("any is: {:?}", comp.eval());
        assert!(comp.eval() == true.into());
    }
}
