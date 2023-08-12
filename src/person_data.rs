use std::{collections::HashMap, hash::Hash, sync::RwLock};

use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

cfg_if! {
if #[cfg(feature = "ssr")] {
    use crate::persistence::SaveInRonFile;

    #[derive(Debug, Default)]
    pub struct PersonStore {
        store: RwLock<PersonIdMap>,
    }

    impl PersonStore {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn new_from_id_map(id_map: PersonIdMap) -> Self {
            Self {
                store: RwLock::new(id_map)
            }
        }

        pub fn add(&self, person: Person) -> PersonId {
            let mut store = self.store.write().unwrap();

            let id = store.next_id;

            store.map.insert(id, person);
            store.next_id = PersonId(id.0 + 1);

            store.save();

            id
        }

        pub fn get_person(&self, id: PersonId) -> Option<Person> {
            self.store.read().unwrap().map.get(&id).cloned()
        }

        pub fn list_names_and_ids(&self) -> Vec<(PersonId, String)> {
            self.store.read().unwrap().map.iter().map(|(id, person)| (*id, person.name.clone())).collect()
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PersonIdMap {
        map: HashMap<PersonId, Person>,
        next_id: PersonId,
    }

    impl Default for PersonIdMap {
        fn default() -> Self {
            Self {
                map: HashMap::new(),
                next_id: PersonId(0),
            }
        }
    }

    // TODO (2023-08-11): Do we maybe want to store each person in their own .RON file, instead of storing the entire store in a single file?
    impl SaveInRonFile for PersonIdMap {
        const FILE_NAME: &'static str = "persons";
    }
}}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default)]
pub struct PersonId(u32);

impl PersonId {
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl leptos_router::IntoParam for PersonId {
    fn into_param(value: Option<&str>, name: &str) -> Result<Self, leptos_router::ParamsError> {
        Ok(Self(u32::into_param(value, name)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
}
