use leptos::*;
use leptos_router::*;
use log::info;
use std::{collections::HashMap, hash::Hash};

use crate::person_data::{Person, PersonId};

pub type PersonCache = StoredValue<(
    Scope,
    HashMap<PersonId, Resource<(), std::result::Result<Person, leptos::ServerFnError>>>,
)>;

pub fn new_cache(cx: Scope) -> PersonCache {
    store_value(cx, (cx, HashMap::new()))
}

pub fn get_person(
    cache: PersonCache,
    id: PersonId,
) -> Resource<(), Result<Person, leptos::ServerFnError>> {
    let maybe_resource = cache.with_value(|persons| persons.1.get(&id).cloned());
    let resource = match maybe_resource {
        Some(resource) => resource,
        None => {
            let context = cache.with_value(|persons| persons.0);
            let resource = create_resource(context, || {}, move |_| request_person(context, id));
            cache.update_value(|cache| {
                cache.1.insert(id, resource);
            });
            resource
        }
    };
    resource
}

#[server(CreatePerson, "/api")]
pub async fn create_person(cx: Scope, name: String) -> Result<(), ServerFnError> {
    use crate::person_data::PersonStore;
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

#[server(RequestPerson, "/api")]
pub async fn request_person(cx: Scope, id: PersonId) -> Result<Person, ServerFnError> {
    use crate::person_data::PersonStore;
    use actix_web::web;
    use leptos_actix::extract;

    info! {"Retrieving person with id: {}", id.raw()};

    extract(cx, move |persons: web::Data<PersonStore>| async move {
        persons.get(id)
    })
    .await
    .and_then(|maybe_person| {
        maybe_person.ok_or_else(|| ServerFnError::ServerError("Person not found".to_string()))
    })
}

#[derive(Params, PartialEq, Eq, Clone)]
struct PersonParams {
    id: PersonId,
}

#[component]
pub fn SinglePerson(cx: Scope, persons: PersonCache) -> impl IntoView {
    // TODO (2023-08-11): https://leptos-rs.github.io/leptos/router/18_params_and_queries.html
    let params = use_params::<PersonParams>(cx);
    let id =
        move || params.with(|params| params.clone().map(|params| params.id).unwrap_or_default());

    let resource = get_person(persons, id());

    view! {cx,
        <Suspense
            fallback=move || view! {cx, <p>"Loading..."</p>}
        >
            {move || resource.read(cx).map(|maybe_person| match maybe_person{
                Ok(person) => view!{cx, <p>{person.name}</p>},
                Err(e) => {
                    let message = format!("Error while loading person: {}", e);
                    view!{cx, <p>{message}</p>}
                }
            })
            }
        </Suspense>
    }
}

#[component]
pub fn PersonsView(cx: Scope) -> impl IntoView {
    let create_person = create_server_multi_action::<CreatePerson>(cx);
    let submissions = create_person.submissions();

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
            // Nested child views appear here.
            <Outlet/>
        </div>
    }
}
