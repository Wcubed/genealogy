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

        pub fn add_with_name(&self, name: String) -> PersonId {
            let mut store = self.store.write().unwrap();

            let id = store.next_id;
            let person = Person { id, name };

            store.map.insert(id, person);
            store.next_id = PersonId(id.0 + 1);

            store.save();

            id
        }

        pub fn get_person(&self, id: PersonId) -> Option<Person> {
            self.store.read().unwrap().map.get(&id).cloned()
        }

        pub fn search_by_names(&self, search_string: &str) -> Vec<(PersonId, String)> {
            let mut names: Vec<_> = self.store.read().unwrap().map.iter().filter(|(_, person)| person.name.contains(search_string)).map(|(id, person)| (*id, person.name.clone())).collect();
            names.sort_by(|a, b| a.1.cmp(&b.1));
            names
        }

        pub fn update_name(&self, id: PersonId, new_name: String) {
            // TODO (2023-08-13): Return an error if the person does not exist.
            let mut store = self.store.write().unwrap();
            store.map.entry(id).and_modify(|person| person.name = new_name);

            store.save();
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
    pub id: PersonId,
    pub name: String,
}
