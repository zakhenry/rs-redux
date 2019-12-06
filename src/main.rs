use std::collections::HashMap;
use std::borrow::Borrow;

#[derive(Debug, Copy, Clone)]
struct Todo {
    id: i32,
    task: i32
}

trait Identifiable {
    fn get_id(&self) -> i32;
}

impl Identifiable for Todo {
    fn get_id(&self) -> i32 {
        self.id
    }
}

#[derive(Debug)]
struct Collection<T: Identifiable + Clone> {
    ids: Vec<i32>,
    entities: HashMap<i32, T>
}

impl<T: Identifiable + Clone> Clone for Collection<T> {
    fn clone(&self) -> Self {
        Collection { ids: self.ids.clone(), entities: self.entities.clone() }
    }
}

impl<T: Identifiable + Clone> Collection<T> {
    fn new() -> Collection<T> {
        Collection { ids: vec![], entities: Default::default() }
    }

    fn add(&self, entity: T) -> Collection<T> {

        let mut new_collection = self.clone();

        let id = entity.get_id();

        new_collection.ids.push(id);
        new_collection.entities.insert(id, entity);

        new_collection
    }

    fn update(&self, entity: T) -> Collection<T> {

        let mut new_collection = self.clone();

        let id = entity.get_id();

        new_collection.entities.insert(id, entity);

        new_collection
    }

    fn remove(&self, id: i32) -> Collection<T> {

        let mut new_collection = self.clone();

        new_collection.ids.remove(new_collection.ids.iter().position(|&e| e == id).expect("Entity should exist!"));
        new_collection.entities.remove(id.borrow());

        new_collection
    }
}

#[derive(Clone, Copy)]
enum EntityAction<T> {
    AddEntity(T),
    RemoveEntity(i32),
    UpdateEntity(T),
}

struct Store<T, P> {
    state: T,
    reducers: Vec<Box<dyn Fn(T, EntityAction<P>) -> T>>
}


#[derive(Clone, Debug)]
struct TodoState {
    todos: Collection<Todo>
}

impl TodoState {
    fn new() -> TodoState {
        TodoState { todos: Collection::new() }
    }
}

impl Store<TodoState, Todo> {

    fn new(state: TodoState) -> Store<TodoState, Todo> {
        Store { state, reducers: vec![] }
    }

    fn register_reducer(&mut self, reducer: Box<dyn Fn(TodoState, EntityAction<Todo>) -> TodoState>) -> &mut Store<TodoState, Todo> {
        self.reducers.push(reducer);
        self
    }

    fn dispatch(&mut self, action: EntityAction<Todo>) {
        self.state = self.reducers.iter().fold(self.state.clone(), |prev_state, reducer| reducer(prev_state, action))
    }

    fn get_state(&self) -> &TodoState {
        self.state.borrow()
    }

}

fn todo_reducer(todo_state: TodoState, action: EntityAction<Todo>) -> TodoState {


    match action {
        EntityAction::AddEntity(todo) => {
            let mut new_state = todo_state.clone();

            new_state.todos = todo_state.todos.add(todo);

            new_state
        },
        EntityAction::UpdateEntity(todo) => {
            let mut new_state = todo_state.clone();

            new_state.todos = todo_state.todos.update(todo);

            new_state
        },
        EntityAction::RemoveEntity(id) => {
            let mut new_state = todo_state.clone();

            new_state.todos = todo_state.todos.remove(id);

            new_state
        },
    }

}


fn main() {

    println!("Hello, redux!");

    let mut store = Store::new(TodoState::new());

    store.register_reducer(Box::new(todo_reducer));

    store.dispatch(EntityAction::AddEntity(Todo { id: 1, task: 42 }));
    println!("State is {:?}", store.get_state());
    store.dispatch(EntityAction::AddEntity(Todo { id: 2, task: 55 }));
    println!("State is {:?}", store.get_state());

    store.dispatch(EntityAction::RemoveEntity(1));
    println!("State is {:?}", store.get_state());

    store.dispatch(EntityAction::UpdateEntity(Todo { id: 2, task: 99 }));

    println!("State is {:?}", store.get_state());
}
