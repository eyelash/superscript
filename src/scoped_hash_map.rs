use std::hash::Hash;

pub struct ScopedHashMap<K, V> {
	scopes: Vec<std::collections::HashMap<K, V>>,
}

impl <K: Hash + Eq, V> ScopedHashMap<K, V> {
	pub fn new() -> Self {
		let mut scopes = Vec::new();
		scopes.push(std::collections::HashMap::new());
		ScopedHashMap {
			scopes,
		}
	}
	pub fn push_scope(&mut self) {
		self.scopes.push(std::collections::HashMap::new());
	}
	pub fn pop_scope(&mut self) {
		self.scopes.pop();
	}
	pub fn get_local<Q: Hash + Eq>(&self, k: &Q) -> Option<&V> where K: std::borrow::Borrow<Q> {
		match self.scopes.last() {
			Some(scope) => scope.get(k),
			None => None,
		}
	}
	pub fn get<Q: Hash + Eq>(&self, k: &Q) -> Option<&V> where K: std::borrow::Borrow<Q> {
		for scope in self.scopes.iter().rev() {
			match scope.get(k) {
				Some(v) => return Some(v),
				None => continue,
			}
		}
		None
	}
	pub fn insert(&mut self, k: K, v: V) -> Option<V> {
		self.scopes.last_mut().unwrap().insert(k, v)
	}
}
