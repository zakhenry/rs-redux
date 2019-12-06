use std::borrow::Borrow;
use std::collections::HashMap;

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


type Reducer<State, Action> = dyn Fn(State, Action) -> State;
type EntityReducer<State, Entity> = Reducer<State, EntityAction<Entity>>;
type Observer<T> = dyn Fn(T);

type Selector<State, T> = dyn Fn(State) -> T;

struct ObserverSelector<State, T> {
    selector: Box<Selector<State, T>>,
    observer: Box<Observer<T>>
}

struct Store<T, P> {
    state: T,
    reducers: Vec<Box<EntityReducer<T, P>>>,
    observers: Vec<ObserverSelector<T, bool>>,
}

// Concrete impl follows

#[derive(Debug, Copy, Clone)]
struct Todo {
    id: i32,
    task: i32
}

#[derive(Clone, Debug)]
struct RootState {
    todos: Collection<Todo>
}

impl RootState {
    fn new() -> RootState {
        RootState { todos: Collection::new() }
    }
}


impl Store<RootState, Todo> {

    fn new(state: RootState) -> Store<RootState, Todo> {
        Store { state, reducers: vec![], observers: vec![] }
    }

    fn register_reducer(&mut self, reducer: Box<EntityReducer<RootState, Todo>>) -> &mut Store<RootState, Todo> {
        self.reducers.push(reducer);
        self
    }

    fn dispatch(&mut self, action: EntityAction<Todo>) {
        self.state = self.reducers.iter().fold(self.state.clone(), |prev_state, reducer| reducer(prev_state, action));

        self.observers.iter().for_each(|so| (so.observer)((so.selector)(self.state.clone())));
    }

    fn get_state(&self) -> &RootState {
        self.state.borrow()
    }

    fn select<T>(&self, selector: Box<Selector<RootState, T>>) -> T {
        selector(self.state.clone())
    }

    fn observe(&mut self, selector: Box<Selector<RootState, bool>>, observer: Box<Observer<bool>>) {
        self.observers.push(ObserverSelector {selector, observer })
    }

}

fn todo_reducer(todo_state: RootState, action: EntityAction<Todo>) -> RootState {


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

fn select_id_2_todo_task_full(state: RootState) -> Option<i32> {

    let collection = state.todos;

    let todo = collection.entities.get(&2);

    match todo {
        None => None,
        Some(t) => Some(t.task),
    }
}

fn test_observer(state: RootState) -> bool {
    match select_id_2_todo_task_full(state) {
        Some(99) => true,
        Some(_) => false,
        None => false
    }
}


fn main() {

    println!("Hello, redux!");

    let mut store = Store::new(RootState::new());

    store.register_reducer(Box::new(todo_reducer));

    store.observe(Box::new(test_observer), Box::new(|v| println!("task 2 is 99! {:?}", v)));

    store.dispatch(EntityAction::AddEntity(Todo { id: 1, task: 42 }));
    println!("State is {:?}", store.get_state());
    store.dispatch(EntityAction::AddEntity(Todo { id: 2, task: 55 }));
    println!("State is {:?}", store.get_state());

    store.dispatch(EntityAction::RemoveEntity(1));
    println!("State is {:?}", store.get_state());

    store.dispatch(EntityAction::UpdateEntity(Todo { id: 2, task: 99 }));

    println!("State select_id_2_todo_task_full is {:?}", store.select(Box::new(select_id_2_todo_task_full)));
}
