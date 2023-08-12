use leptos::*;
use leptos_router::*;
use log::info;

use crate::person_data::{Person, PersonId};

#[server(CreatePerson, "/api")]
pub async fn create_person(cx: Scope, name: String) -> Result<(), ServerFnError> {
    use crate::person_data::PersonStore;
    use actix_web::web;
    use leptos_actix::extract;
    use log::info;

    extract(cx, |persons: web::Data<PersonStore>| async move {
        let person = Person { name };

        info!("New person: {:?}", person);

        persons.add(person);
    })
    .await
}

#[server(GetPerson, "/api")]
pub async fn get_person(cx: Scope, id: PersonId) -> Result<Person, ServerFnError> {
    use crate::person_data::PersonStore;
    use actix_web::web;
    use leptos_actix::extract;

    info! {"Retrieving person with id: {}", id.raw()};

    extract(cx, move |persons: web::Data<PersonStore>| async move {
        persons.get_person(id)
    })
    .await
    .and_then(|maybe_person| {
        maybe_person.ok_or_else(|| ServerFnError::ServerError("Person not found".to_string()))
    })
}

#[server(GetPersonList, "/api")]
pub async fn get_persons_list(cx: Scope) -> Result<Vec<(PersonId, String)>, ServerFnError> {
    use crate::person_data::PersonStore;
    use actix_web::web;
    use leptos_actix::extract;

    extract(cx, move |persons: web::Data<PersonStore>| async move {
        persons.list_names_and_ids()
    })
    .await
}

#[derive(Params, PartialEq, Eq, Clone)]
struct PersonParams {
    id: PersonId,
}

#[component]
pub fn SinglePerson(cx: Scope) -> impl IntoView {
    // TODO (2023-08-11): https://leptos-rs.github.io/leptos/router/18_params_and_queries.html
    let params = use_params::<PersonParams>(cx);
    let id =
        move || params.with(|params| params.clone().map(|params| params.id).unwrap_or_default());

    let person_resource = create_resource(cx, id, move |id| get_person(cx, id));

    view! {cx,
        <Transition
            fallback=move || view! {cx, <p>"Loading..."</p>}
        >
            {move || person_resource.read(cx).map(|maybe_person| match maybe_person{
                Ok(person) => view!{cx, <p>{person.name}</p>},
                Err(e) => {
                    let message = format!("Error while loading person: {}", e);
                    view!{cx, <p>{message}</p>}
                }
            })
            }
        </Transition>
    }
}

#[component]
pub fn PersonsView(cx: Scope) -> impl IntoView {
    let create_person = create_server_multi_action::<CreatePerson>(cx);
    let person_list = create_resource(
        cx,
        move || create_person.version().get(),
        move |_| get_persons_list(cx),
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
            <Transition
                fallback=move || view! {cx, <p>"Loading..."</p>}
            >
                {move || person_list.read(cx).map(|maybe_persons| match maybe_persons{
                    Ok(persons) => {
                        view!{cx,
                            <ul>
                                <For
                                    each=move || persons.clone().into_iter()
                                    key=|person| person.0
                                    view=move|cx, person| {
                                        view!{cx,
                                            <li><A href={person.0.raw().to_string()}>{person.1.to_string()}</A></li>
                                        }
                                    }
                                >
                                </For>
                            </ul>
                        }.into_view(cx)
                    },
                    Err(e) => {
                        let message = format!("Error while loading person: {}", e);
                        view!{cx, <p>{message}</p>}.into_view(cx)
                    }
                })
                }
            </Transition>

            // Nested child view appears here.
            <Outlet/>
        </div>
    }
}
