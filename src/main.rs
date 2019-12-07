use std::borrow::Borrow;
use std::collections::HashMap;

trait Identifiable {
    fn get_id(&self) -> i32;
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

    fn add(self, entity: &T) -> Collection<T> {

        let mut new_collection = self.clone();

        let id = entity.get_id();

        new_collection.ids.push(id);
        new_collection.entities.insert(id, entity.clone());

        new_collection
    }

    fn update(self, entity: &T) -> Collection<T> {

        let id = entity.get_id();

        let mut entities = self.entities.clone();
        entities.insert(id, entity.clone());

        Collection {
            ids: self.ids, // ids here is moved, not copied
            entities
        }
    }

    fn remove(self, id: &i32) -> Collection<T> {

        let mut new_collection = self.clone();

        new_collection.ids.remove(new_collection.ids.iter().position(|&e| e == *id).expect("Entity should exist!"));
        new_collection.entities.remove(id);

        new_collection
    }
}

#[derive(Clone)]
enum EntityAction<T: Identifiable> {
    AddEntity(T),
    RemoveEntity(i32),
    ReplaceEntity(T),
}


type Reducer<State, Action> = dyn Fn(State, &Action) -> State;
type Observer<T> = dyn Fn(T);

type Selector<State, T> = dyn Fn(State) -> T;

struct ObserverSelector<State, T> {
    selector: Box<Selector<State, T>>,
    observer: Box<Observer<T>>
}

struct Store<T, A> {
    state: T,
    reducers: Vec<Box<Reducer<T, A>>>,
    observers: Vec<ObserverSelector<T, bool>>,
}

impl<State, Action> Store<State, Action> where State: Clone, Action: Clone {

    fn new(state: State) -> Self {
        Store { state, reducers: vec![], observers: vec![] }
    }

    fn register_reducer(&mut self, reducer: Box<Reducer<State, Action>>) -> &mut Self {
        self.reducers.push(reducer);
        self
    }

    fn dispatch(&mut self, action: Action) {
        self.state = self.reducers.iter().fold(self.state.clone(), |prev_state, reducer| reducer(prev_state, &action));

        self.observers.iter().for_each(|so| (so.observer)((so.selector)(self.state.clone())));
    }

    fn get_state(&self) -> &State {
        self.state.borrow()
    }

    fn select<T>(&self, selector: Box<Selector<State, T>>) -> T {
        selector(self.state.clone())
    }

    fn observe(&mut self, selector: Box<Selector<State, bool>>, observer: Box<Observer<bool>>) {
        self.observers.push(ObserverSelector {selector, observer })
    }

}

fn entity_reducer<Entity: Identifiable + Clone>(entity_state: Collection<Entity>, action: &EntityAction<Entity>) -> Collection<Entity> {

    match action {
        EntityAction::AddEntity(entity) => entity_state.add(entity),
        EntityAction::ReplaceEntity(entity) => entity_state.update(entity),
        EntityAction::RemoveEntity(id) => entity_state.remove(id),
    }

}


// Concrete impl follows

#[derive(Debug, Clone)]
struct Todo {
    id: i32,
    task: String,
    done: bool,
}

impl Todo {
    fn new(id: i32, task: &str) -> Todo {
        Todo { task: String::from(task), id, done: false }
    }
}


#[derive(Clone)]
enum TodoAction {
    Entity(EntityAction<Todo>),
    MarkDone(i32, bool),
    ChangeText(i32, String)
}


impl Identifiable for Todo {
    fn get_id(&self) -> i32 {
        self.id
    }
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

fn todo_reducer(todo_state: RootState, action: &TodoAction) -> RootState {

    match action {
        TodoAction::Entity(x) => {
            let mut new_state = todo_state.clone();

            new_state.todos = entity_reducer(todo_state.todos, &x);

            new_state
        },
        TodoAction::MarkDone(id, done) => {
            let mut new_state = todo_state.clone();

            let mut todo = new_state.todos.entities.get_mut(id).expect("Cannot mark missing todo as done");

            todo.done = *done;

            new_state
        }
        TodoAction::ChangeText(id, text) => {

            let mut new_state = todo_state.clone();
            let mut todo = new_state.todos.entities.get_mut(id).expect("Cannot update text of missing todo");

            todo.task = text.to_owned();

            new_state

        }
    }

}

fn select_id_2_todo_task_done(state: RootState) -> Option<bool> {

    let collection = state.todos;

    let todo = collection.entities.get(&2);

    match todo {
        None => None,
        Some(t) => Some(t.done),
    }
}

fn test_observer(state: RootState) -> bool {
    match select_id_2_todo_task_done(state) {
        Some(_) => true,
        None => false
    }
}


fn main() {

    println!("Hello, redux!");

    let mut store: Store<RootState, TodoAction> = Store::new(RootState::new());

    store.register_reducer(Box::new(todo_reducer));

    store.observe(Box::new(test_observer), Box::new(|v| println!("task 2 is set! {:?}", v)));

    store.dispatch(TodoAction::Entity(EntityAction::AddEntity(Todo::new(1, "understand &references") )));
    println!("State is {:?}", store.get_state());
    store.dispatch(TodoAction::Entity(EntityAction::AddEntity(Todo::new(2, "get good") )));
    store.dispatch(TodoAction::Entity(EntityAction::AddEntity(Todo::new(3, "understand 'lifetimes") )));
    println!("State is {:?}", store.get_state());

    store.dispatch(TodoAction::MarkDone(1, true));
    store.dispatch(TodoAction::MarkDone(2, true));
    println!("State is {:?}", store.get_state());
    store.dispatch(TodoAction::Entity(EntityAction::RemoveEntity(1)));
    println!("State is {:?}", store.get_state());

    store.dispatch(TodoAction::Entity(EntityAction::ReplaceEntity(Todo::new(2, "get gooder") )));
    store.dispatch(TodoAction::Entity(EntityAction::ReplaceEntity(Todo::new(2, "get goodest") )));
    store.dispatch(TodoAction::Entity(EntityAction::RemoveEntity(2)));
    store.dispatch(TodoAction::Entity(EntityAction::AddEntity(Todo::new(2, "get good") )));

    store.dispatch(TodoAction::ChangeText(2, String::from("git gud")));

    println!("State is {:?}", store.get_state());

    println!("State select_id_2_todo_task_full is {:?}", store.select(Box::new(select_id_2_todo_task_done)));
}
