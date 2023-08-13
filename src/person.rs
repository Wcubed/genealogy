use cfg_if::cfg_if;
use leptos::*;
use leptos_router::*;
use log::info;

cfg_if! {
if #[cfg(feature = "ssr")] {
    use crate::person_data::PersonStore;
    use actix_web::web;
    use leptos_actix::extract;
}}

use crate::{
    error_template::ErrorTemplate,
    person_data::{Person, PersonId},
};

#[server(CreatePerson, "/api")]
pub async fn create_person(cx: Scope, name: String) -> Result<(), ServerFnError> {
    extract(cx, |persons: web::Data<PersonStore>| async move {
        persons.add_with_name(name);
    })
    .await
}

#[server(UpdatePersonName, "/api")]
pub async fn update_person_name(
    cx: Scope,
    id: PersonId,
    new_name: String,
) -> Result<(), ServerFnError> {
    extract(cx, move |persons: web::Data<PersonStore>| async move {
        persons.update_name(id, new_name);
    })
    .await
}

#[server(GetPerson, "/api")]
pub async fn get_person(cx: Scope, id: PersonId) -> Result<Person, ServerFnError> {
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

    let edit_name_action = create_server_action::<UpdatePersonName>(cx);
    let person_resource = create_resource(
        cx,
        move || (id(), edit_name_action.version().get()),
        move |_| get_person(cx, id()),
    );

    view! {cx,
        <Transition
            fallback=move || view! {cx, <p>"Loading..."</p>}
        >
            {move || person_resource.read(cx).map(|maybe_person| match maybe_person {
                Ok(person) => {
                    let (person, _) = create_signal(cx, person);
                    view!{cx, <SinglePersonView person=person edit_name=edit_name_action/>}
                },
                Err(e) => view!{cx, <ErrorTemplate error=e/>},
            })}
        </Transition>
    }
}

#[component]
pub fn SinglePersonView(
    cx: Scope,
    person: ReadSignal<Person>,
    edit_name: Action<UpdatePersonName, Result<(), ServerFnError>>,
) -> impl IntoView {
    let (editing_name, set_editing_name) = create_signal(cx, false);

    view! {cx,
        <Show
            when=editing_name
            fallback=move|cx| view!{cx, <h1 on:click=move |_|{set_editing_name.set(true)}>{person().name}</h1>}
        >
            <ActionForm action=edit_name>
                <input type="hidden" name="id" value={person().id.raw()}/>
                <input type="text" placeholder="name" name="new_name" value={person().name} autofocus=true/>
                <input type="submit" value="Ok" title="[Enter]"/>
                <input type="button" value="Cancel" on:click=move |_| {
                        set_editing_name.set(false)
                    }
                />
            </ActionForm>
        </Show>
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
        <div class="content-with-sidebar">
        <div class="sidebar">
            <MultiActionForm action=create_person>
                <input type="text" placeholder="name" name="name"/>
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
        </div>
        <div class="main-content">
            // Nested child view appears here.
            <Outlet/>
        </div>
        </div>
    }
}
