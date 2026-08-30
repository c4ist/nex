//! lexical scopes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// one scope plus a link to the one enclosing it.
///
/// closures capture an `Rc<Scope>`, so a scope outlives the call that
/// created it if a function defined inside it escapes.
pub struct Scope {
    bindings: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Scope>>,
}

impl Scope {
    pub fn global() -> Rc<Scope> {
        Rc::new(Scope {
            bindings: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    /// a new scope nested inside `parent`
    pub fn child(parent: &Rc<Scope>) -> Rc<Scope> {
        Rc::new(Scope {
            bindings: RefCell::new(HashMap::new()),
            parent: Some(Rc::clone(parent)),
        })
    }

    /// introduces a binding here, shadowing any outer one of the same name
    pub fn define(&self, name: impl Into<String>, value: Value) {
        self.bindings.borrow_mut().insert(name.into(), value);
    }

    /// walks outwards until the name is found
    pub fn lookup(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.borrow().get(name) {
            return Some(value.clone());
        }
        self.parent.as_ref()?.lookup(name)
    }

    /// assigns to an existing binding wherever it lives, so `x = 1` inside
    /// a block updates the outer `x` rather than shadowing it.
    ///
    /// returns `false` if the name was never defined; assignment doesn't
    /// create bindings, `let` does.
    pub fn assign(&self, name: &str, value: Value) -> bool {
        if let Some(slot) = self.bindings.borrow_mut().get_mut(name) {
            *slot = value;
            return true;
        }
        match &self.parent {
            Some(parent) => parent.assign(name, value),
            None => false,
        }
    }

    /// whether the name is bound in this scope, ignoring enclosing ones
    pub fn defined_locally(&self, name: &str) -> bool {
        self.bindings.borrow().contains_key(name)
    }
}
