use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryState<T> {
	Loading,
	Data(T),
	Error(String),
}
