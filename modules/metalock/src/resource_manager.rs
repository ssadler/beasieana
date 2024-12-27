
use crate::{resources::*, types::*};



#[derive(Clone, PartialEq, Eq)]
pub struct Resources {
    resources: Vec<(ResourceType, Vec<(String, ResourceDataOrPtr)>)>
}


impl Resources {
    pub fn get<T: IsResourceData>(&self, name: String) -> Option<T> {
        let typ = T::to_resource_schema().encode();
        self._get(&ResourceId(typ, name)).map(
            |b| ResourceDataDeserialize::rd_deserialize(&mut &**b).expect("Could not deserialize")
        )
    }
    pub fn set<T: IsResourceData>(&mut self, name: String, data: T) {
        let t = T::to_resource_schema().encode();
        let d = data.rd_serialize();
        self._set(ResourceId(t, name), ResourceDataOrPtr::Data(d), Self::SET_INSERT);
    }
    pub fn set_with_schema(&mut self, schema: &ResourceSchema, name: String, data: ResourceData) {
        assert!(validate_resource_data(&"", schema, &data).is_ok(), "Resources::set: invalid data");
        self._set(ResourceId(schema.encode(), name), data.into(), Self::SET_INSERT | Self::SET_UPDATE);
    }
    const SET_INSERT: u8 = 1;
    const SET_UPDATE: u8 = 2;

    fn _set(&mut self, ResourceId(typ, name): ResourceId, data: ResourceDataOrPtr, op: u8) -> SetResult {
        if let Some((_, resources)) = self.resources.iter_mut().find(|r| r.0 == typ) {
            if let Some(v) = resources.iter_mut().find(|f| f.0 == name) {
                if op & 1 > 0 {
                    return SetResult::Replaced(std::mem::replace(&mut v.1, data));
                }
            }
            if op & 2 > 0 {    
                resources.push((name, data.into()));
                return SetResult::Inserted;
            }
        } else {
            if op & 2 > 0 {    
                self.resources.push((typ, vec![(name, data.into())]));
                return SetResult::Inserted;
            }
        }
        SetResult::Noop
    }
    //pub fn take(&mut self, ResourceId(typ, name): ResourceId) -> Option<ResourceData> {
    //    if let Some(idxa) = self.resources.iter().position(|r| r.0 == typ) {
    //        let d = &mut self.resources[idxa].1;
    //        if let Some(idxb) = d.iter().position(|f| f.0 == name) {
    //            let o = Some(d.remove(idxb).1);
    //            if d.len() == 0 {
    //                self.resources.remove(idxa);
    //            }
    //            return o;
    //        }
    //    }
    //    None
    //}
    //

    /*
     * Upgrade allows additional fields to be added to a schema,
     * for example, if you have the schema:
     *
     * { members: [{ name: String, age: u8 }] }
     *
     * You can upgrade it to a compatible schema:
     *   
     * { members: [{ name: String, age: u8, balance: u64 }] }
     *
     * And the old key will point to the new key for backwards compatability
     */
    pub fn upgrade(&mut self, old_id: ResourceId, new_id: ResourceId) {
        assert!(
            schema_is_superset(&*old_id.0.to_schema(), &*new_id.0.to_schema()),
            "Resources::upgrade: not superset"
        );

        let ptr = ResourceDataOrPtr::Ptr(new_id.clone());
        let old = self._set(old_id, ptr, Self::SET_UPDATE);
        let old = match old {
            SetResult::Replaced(d) => d,
            _ => panic!("Resources::upgrade: old does not exist")
        };

        let r = self._set(new_id, old, Self::SET_INSERT);
        assert!(r == SetResult::Inserted, "Resources::upgrade: new already exists");
    }

    fn _get(&self, id: &ResourceId) -> Option<&Vec<u8>> {
        let r = &self.resources.iter().find(|r| r.0 == id.0)?.1.iter().find(|f| f.0 == id.1)?.1;
        match r {
            ResourceDataOrPtr::Data(d) => Some(d),
            ResourceDataOrPtr::Ptr(p) => self._get(p)
        }
    }
}


#[derive(PartialEq, Eq)]
enum SetResult {
    Replaced(ResourceDataOrPtr),
    Inserted,
    Noop
}
