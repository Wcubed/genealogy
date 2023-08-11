use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::RwLock,
};

use cfg_if::cfg_if;
use leptos::*;
use leptos_router::MultiActionForm;
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

        pub fn get_all(&self) -> PersonMap {
            self.store.read().unwrap().map.clone()
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PersonIdMap {
        map: PersonMap,
        next_id: PersonId,
    }

    impl Default for PersonIdMap {
        fn default() -> Self {
            Self {
                map: PersonMap(HashMap::new()),
                next_id: PersonId(0),
            }
        }
    }

    // TODO (2023-08-11): Do we maybe want to store each person in their own .RON file, instead of storing the entire store in a single file?
    impl SaveInRonFile for PersonIdMap {
        const FILE_NAME: &'static str = "persons";
    }
}}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersonMap(HashMap<PersonId, Person>);

impl Deref for PersonMap {
    type Target = HashMap<PersonId, Person>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PersonMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersonId(u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
}

#[server(CreatePerson, "/api")]
pub async fn create_person(cx: Scope, name: String) -> Result<(), ServerFnError> {
    use actix_web::web;
    use leptos_actix::extract;
    use log::info;

    // fake API delay
    std::thread::sleep(std::time::Duration::from_millis(2000));

    extract(cx, |persons: web::Data<PersonStore>| async move {
        let person = Person { name };

        info!("New person: {:?}", person);

        persons.add(person);
    })
    .await
}

#[server(GetPersons, "/api")]
pub async fn get_persons(cx: Scope) -> Result<PersonMap, ServerFnError> {
    use actix_web::web;
    use leptos_actix::extract;

    extract(cx, |persons: web::Data<PersonStore>| async move {
        persons.get_all()
    })
    .await
}

#[component]
pub fn PersonsView(cx: Scope) -> impl IntoView {
    let create_person = create_server_multi_action::<CreatePerson>(cx);
    let submissions = create_person.submissions();

    let persons = create_resource(
        cx,
        move || create_person.version().get(),
        move |_| get_persons(cx),
    );

    view! {
        cx,
        <div>
            <MultiActionForm action=create_person>
                <label>
                    Name:
                    <input type="text" name="name"/>
                </label>
                <input type="submit" value="Create"/>
            </MultiActionForm>
            <Transition fallback=move || view! {cx, <p>"Loading..."</p>}>
                {move || {
                    let existing_persons = {
                                persons.read(cx).map(move |persons| match persons {
                                    Err(e) => {
                                        view! { cx, <pre class="error">"Server Error: " {e.to_string()}</pre>}.into_view(cx)
                                    },
                                    Ok(persons) => {
                                        persons.iter().map(move |(_id, person)| {
                                        view!{
                                            cx,
                                            <li>
                                                {person.name.clone()}
                                            </li>
                                        }
                                }).collect_view(cx)
                            }
                        })
                    };
                    let pending_persons = move || {
                        submissions
                        .get()
                        .into_iter()
                        .filter(|submission| submission.pending().get())
                        .map(|submission| {
                            view! {
                                cx,
                                <li class="pending">{move || submission.input.get().map(|person| person.name) }</li>
                            }
                        })
                        .collect_view(cx)
                    };

                    view!{
                        cx,
                        <ul>
                            {existing_persons}
                            {pending_persons}
                        </ul>
                    }
                }}
            </Transition>
        </div>
    }
}
