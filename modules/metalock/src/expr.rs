
// You want a function to be able to return a value, so you need a trait
// every node has inputs and an output (and maybe side effects)
// So every node has inputs, outputs, and an instruction
// and all nodes implement a common interface
// which implements all the data methods via impl specialisations

// Everything implemented with macros, for example, every op has an impl to return the
// opcode ID

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::impl_deref;
use crate::parse::rdd_deserialize;
use crate::parse::take;
use crate::types::*;
use crate::encode::*;


#[derive(Debug)]
#[repr(u8)]
pub enum OP {
    EQ = 1,
    LEN = 2,
    ADD = 3,
    //GT,
    //LT,
    //GTE,
    //LTE,
    VAL = 4,
    EACH = 5,
    //AND,
    //OR,
    NOT = 6,
    SLICE = 7,
    FILTER = 8,
    //REDUCE,
    //GET,
    //HAS,
    //MIN,
    //MAX,
    MAP = 0x40,
    ALL = 0x41,
    ANY = 0x42,
    //
    INDEX = 0x50,
    //BIT_AND,
    //BIT_OR,
    //BIT_NOT,
    //POW,
    //SHL,
    //SHR,
    //MUL,
    //SUB,
    //DIV,

    REF = 0x60,
    FUN1 = 0x70,
}
impl Into<OP> for u8 {
    fn into(self) -> OP {
        unsafe { std::mem::transmute::<u8, OP>(self) }
    }
}


trait Opcode {
    const OP: OP;
}

pub trait R<OUT> {}

pub trait Encode: Opcode {
    fn eval(&self) -> RD {
        let o = self.encode();
        Evaluator::new(&mut o.as_ref()).eval()
    }
    fn encode(&self) -> Vec<u8> {
        vec![Self::OP as u8]
    }
}

macro_rules! impl_opcode {
    ($op:expr,$type:ident,[$($alias:ident: [$($constraints:tt)*]),*]) => {
        impl<$($alias: $($constraints)*),*> Opcode for $type<$($alias),*> {
            const OP: OP = $op;
        } } }
macro_rules! impl_R {                                                      
    ($ret:ty,$type:ident,[$($alias:ident: [$($constraints:tt)*]),*]) => { 
        impl<$($alias: $($constraints)*),*> R<$ret> for $type<$($alias),*> {}
    } }
macro_rules! fn_encode {
    ([$($field:tt),+]) => {
        fn encode(&self) -> Vec<u8> {
            let mut out = vec![Self::OP as u8];
            $(out.extend(self.$field.encode());)*
            out
        } } }
macro_rules! impl_encode {
    ($name:ident, [$($field:tt),+], [$($a0:ident: [$($c0:tt)*]),*], [$($a1:ident: [$($c1:tt)*]),*]) => {
        impl<$($a0: $($c0)*,)* $($a1: $($c1)* + Encode),*> Encode for $name<$($a0,)* $($a1),*> {
            fn_encode!([$($field),*]);
        }
    };
}


#[derive(Clone, Copy)]
pub struct Equals<T, A: R<T>, B: R<T>>(pub A, pub B, pub PhantomData<T>);
impl_opcode!(OP::EQ, Equals, [T: [], A: [R<T>], B: [R<T>]]);
impl_R!(bool, Equals, [T: [], A: [R<T>], B: [R<T>]]);
impl_encode!(Equals, [0,1], [T: []], [A: [R<T>], B: [R<T>]]);

#[derive(Clone, Copy)]
struct Add<T: std::ops::Add, A: R<T>, B: R<T>>(pub A, pub B, pub PhantomData<T>);
impl_opcode!(OP::ADD, Add, [T: [std::ops::Add], A: [R<T>], B: [R<T>]]);
fn run_add(a: RD, b: RD) -> u64 {
    match (a, b) {
        (RD::U8(a), RD::U8(b)) => ((a+b) as u64).into(),
        (RD::U16(a), RD::U16(b)) => ((a+b) as u64).into(),
        (RD::U32(a), RD::U32(b)) => ((a+b) as u64).into(),
        (RD::U64(a), RD::U64(b)) => ((a+b) as u64).into(),
        _ => panic!("fs")
    }
}
impl_R!(u64,Add,[T: [std::ops::Add], A: [R<T>], B: [R<T>]]);
impl_encode!(Add, [0,1], [T: [std::ops::Add]], [A: [R<T>], B: [R<T>]]);


pub trait HasLen { }
impl<I> HasLen for Vec<I> { }
impl<I> HasLen for BTreeMap<String, I> { }
impl HasLen for Buffer { }
impl HasLen for String { }

pub struct Length<I: HasLen, A: R<I>>(pub A, pub PhantomData<I>);
impl_opcode!(OP::LEN, Length, [I: [HasLen], A: [R<I>]]);
fn run_length(a: RD) -> u16 {
    (match a {
        RD::String(s) => s.len(),
        RD::Map(s) => s.len(),
        RD::List(s) => s.len(),
        RD::Buffer(s) => s.len(),
        _ => panic!("Length: type mismatch")
    }) as u16
}
impl_R!(u16,Length,[I: [HasLen], A: [R<I>]]);
impl_encode!(Length, [0], [T: [HasLen]], [A: [R<T>]]);

pub struct Not<A: R<bool>>(pub A);
impl_opcode!(OP::NOT, Not, [A: [R<bool>]]);
impl_R!(bool,Not,[A: [R<bool>]]);
impl_encode!(Not, [0], [], [A: [R<bool>]]);


#[derive(Clone)]
pub struct Val<A>(pub RD, pub PhantomData<A>);
impl_deref!([A], Val<A>, RD, 0);
impl_opcode!(OP::VAL, Val, [A: []]);
impl<A: Clone> R<A> for Val<A> {}
impl<A: IsResourceData> Val<A> {
    pub fn new(a: A) -> Val<A> {
        Val(a.into(), ph())
    }
}
impl<A: IsResourceData> Encode for Val<A> {
    fn encode(&self) -> Vec<u8> {
        let mut out = vec![Self::OP as u8];
        out.extend(A::to_resource_schema().encode().0);
        out.extend(self.0.rd_serialize());
        out
    }
}
impl<A: IsResourceData> From<A> for Val<A> {
    fn from(value: A) -> Self {
        Val::new(value)
    }
}















pub fn ph<T: Default>() -> T { Default::default() }

#[derive(Clone, Debug)]
pub struct Ref<T>(u32, PhantomData<T>);
impl<T> Ref<T> {
    fn new() -> Self {
        static mut PREV_REF_ID: u32 = 0;
        unsafe {
            PREV_REF_ID += 1;
            Ref(PREV_REF_ID, ph())
        }
    }
}
impl_opcode!(OP::REF, Ref, [T: []]);
impl<T> R<T> for Ref<T> {}
impl<T> Encode for Ref<T> {
    fn encode(&self) -> Vec<u8> {
        let mut out = vec![Self::OP as u8];
        out.extend(self.0.to_le_bytes());
        out
    }
}

#[derive(Clone, Debug)]
pub struct Fun1<I, T: Clone, O: R<T>>(Ref<I>, O, PhantomData<(I, T)>);
impl_opcode!(OP::FUN1, Fun1, [I: [], T: [Clone], O: [R<T>]]);
impl_encode!(Fun1, [0, 1], [I: [], T: [Clone]], [O: [R<T>]]);
impl<I: Clone, T: Clone, O: R<T>> Fun1<I, T, O> {
    pub fn new<F: Fn(Ref<I>) -> O>(f: F) -> Self {
        let r = Ref::new();
        Self(r.clone(), f(r), ph())
    }
}







#[derive(Clone)]
pub struct All<Item, In: R<Vec<Item>>, Body: R<bool>>(pub In, pub Fun1<Item, bool, Body>);
impl<Item: Clone, In: R<Vec<Item>>, Body: R<bool>> All<Item, In, Body> {
    pub fn new<F: Fn(Ref<Item>) -> Body>(vec: In, f: F) -> All<Item, In, Body> {
        All(vec, Fun1::new(f))
    }
}
impl_opcode!(OP::ALL, All, [Item: [], In: [R<Vec<Item>>], Body: [R<bool>]]);
impl_R!(bool, All, [Item: [], In: [R<Vec<Item>>], Body: [R<bool>]]);
impl_encode!(All, [0, 1], [Item: []], [In: [R<Vec<Item>>], Body: [R<bool>]]);

#[derive(Clone)]
pub struct Any<Item, In: R<Vec<Item>>, Body: R<bool>>(pub In, pub Fun1<Item, bool, Body>);
impl<Item: Clone, In: R<Vec<Item>>, Body: R<bool>> Any<Item, In, Body> {
    pub fn new<F: Fn(Ref<Item>) -> Body>(vec: In, f: F) -> Any<Item, In, Body> {
        Any(vec, Fun1::new(f))
    }
}
impl_opcode!(OP::ANY, Any, [Item: [], In: [R<Vec<Item>>], Body: [R<bool>]]);
impl_R!(bool, Any, [Item: [], In: [R<Vec<Item>>], Body: [R<bool>]]);
impl_encode!(Any, [0, 1], [Item: []], [In: [R<Vec<Item>>], Body: [R<bool>]]);



#[derive(Clone)]
pub struct Map<I: Clone, O: Clone, In: R<Vec<I>>, Body: R<O>>(pub In, pub Fun1<I, O, Body>);
impl<I: Clone, O: Clone, In: R<Vec<I>>, Body: R<O>> Map<I, O, In, Body> {
    pub fn new<F: Fn(Ref<I>) -> Body>(vec: In, f: F) -> Map<I, O, In, Body> {
        Map(vec, Fun1::new(f))
    }
}
impl_opcode!(OP::MAP, Map, [I: [Clone], O: [Clone], In: [R<Vec<I>>], Body: [R<O>]]);
impl_R!(O, Map, [I: [Clone], O: [Clone], In: [R<Vec<I>>], Body: [R<O>]]);
impl_encode!(Map, [0, 1], [I: [Clone], O: [Clone]], [In: [R<Vec<I>>], Body: [R<O>]]);




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let a = Val::new(1 as u64);
        let b = Val::new(2 as u64);
        let c = Add(a, b.clone(), ph());
        let d = Add(c.clone(), c.clone(), ph());
        let o = d.encode();
        let r2 = Evaluator::new(&mut o.as_ref()).eval();
        println!("r: {:?}", r2);
    }

    #[test]
    fn test_all() {
        let data: Val<Vec<Vec<u8>>> = Val::new(
            vec![
                vec![1,2],
                vec![1,2,3,4],
            ]
        );

        let o = All::new(data, |r| {
            Not(Equals(Length(r, ph()), Val(RD::U32(0), ph()), ph()))
        }).encode();
        let r = Evaluator::new(&mut o.as_ref()).eval();
        println!("r: {:?}", r);
    }
}




struct Evaluator<'a, 'b>(Buf<'a, 'b>, HashMap<u32, RD>);
impl<'a, 'b> Evaluator<'a, 'b> {
    pub fn new(buf: Buf<'a, 'b>) -> Evaluator<'a, 'b> {
        Self(buf, Default::default())
    }
}

impl<'a, 'b> Evaluator<'a, 'b> {
    fn eval(&mut self) -> RD {
        let op = self.take_op();
        match op {

            OP::EQ => { // Equals
                (self.eval() == self.eval()).into()
            },
            OP::LEN => {
                run_length(self.eval()).into()
            },
            OP::ADD => {
                run_add(self.eval(), self.eval()).into()
            },

            //
            OP::MAP => RD::List(self.fn_map()),
            OP::ALL => self.fn_map().into_iter().all(|rd| rd == RD::Bool(true)).into(),
            OP::ANY => self.fn_map().into_iter().any(|rd| rd == RD::Bool(true)).into(),

            //
            OP::NOT => (self.eval() == false.into()).into(),

            //
            OP::VAL => {
                let rs = ResourceSchema::_decode(self.0).expect("failed reading schema");
                rdd_deserialize(&rs, self.0).expect("failed reading data")
            },
            OP::REF => {
                // Ref needs to return data from a register
                let ref_id = u32::rd_deserialize(self.0).expect("failed reading ref id");
                self.1[&ref_id].clone()
            },

            _ => {
                println!("Got opcode: {:?}", op);
                panic!("Invalid opcode")
            }
        }
    }

    fn fork(&mut self) -> RD {
        let buf = *self.0;
        let out = self.eval();
        *self.0 = buf;
        return out;
    }

    fn fn_map(&mut self) -> Vec<RD>
    {
        let vec = match self.eval() {
            RD::List(vec) => vec,
            _ => panic!("OP:ALL: invalid type")
        };

        // 2 redundant opcodes
        self.take_op();
        self.take_op();

        let ref_id = u32::rd_deserialize(self.0).expect("failed reading ref id");

        let f = |item| {
            self.1.insert(ref_id, item);
            self.fork()
        };

        vec.into_iter().map(f).collect()
    }

    fn take_op(&mut self) -> OP {
        take::<1>(self.0).expect("failed to read opcode")[0].into()
    }
}

