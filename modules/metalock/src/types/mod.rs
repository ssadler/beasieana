
use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::{encode::ResourceDataSerialize, impl_deref};

mod utils;

pub type Buf<'a, 'b> = &'a mut &'b [u8];


#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ResourceSchema(pub(crate) RS);
impl_deref!([], ResourceSchema, RS, 0);
impl std::fmt::Debug for ResourceSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RS {                                                                   
    U8,
    U16,
    U32,
    U64,
    U128,
    Bool,
    String,
    Buffer,
    Pubkey,
    Option(Box<ResourceSchema>),                                                
    List(Box<ResourceSchema>),                                        
    Map(Box<ResourceSchema>),                                                   
    Struct(Vec<(String, ResourceSchema)>),                                      
}








#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ResourceType(pub(crate) Vec<u8>);

impl ResourceType {
    pub fn to_schema(&self) -> ResourceSchema {
        ResourceSchema::decode(self).expect("ResourceType::to_schema failed?")
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RD {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bool(bool),
    String(String),
    Buffer(Buffer),
    Pubkey(Pubkey),
    Option(Box<Option<RD>>),
    List(Vec<RD>),
    Map(BTreeMap<String, RD>),
    Struct(Vec<(String, RD)>),
}

pub type ResourceData = RD;

pub trait ResourceDataDeserialize: Sized {
    fn rd_deserialize(buf: Buf) -> std::result::Result<Self, String>;
}

pub trait IsResourceData: Into<ResourceData> + ResourceDataSerialize + ResourceDataDeserialize {
    fn to_resource_schema() -> ResourceSchema;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Buffer(pub Vec<u8>);
impl_deref!([], Buffer, Vec<u8>, 0);





#[derive(Clone, PartialEq, Eq)]
pub enum ResourceDataOrPtr {
    Data(Vec<u8>),
    Ptr(ResourceId)
}

impl Into<ResourceDataOrPtr> for ResourceData {
    fn into(self) -> ResourceDataOrPtr {
        ResourceDataOrPtr::Data(self.rd_serialize())
    }
}
impl Into<ResourceDataOrPtr> for ResourceId {
    fn into(self) -> ResourceDataOrPtr {
        ResourceDataOrPtr::Ptr(self)
    }
}



#[derive(Clone, PartialEq, Eq)]
pub struct ResourceId(pub ResourceType, pub String);




