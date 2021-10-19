//! Module containing error types used in Garage
use std::fmt;
use std::io;

use err_derive::Error;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::data::*;

/// Regroup all Garage errors
#[derive(Debug, Error)]
pub enum Error {
	#[error(display = "IO error: {}", _0)]
	Io(#[error(source)] io::Error),

	#[error(display = "Hyper error: {}", _0)]
	Hyper(#[error(source)] hyper::Error),

	#[error(display = "HTTP error: {}", _0)]
	Http(#[error(source)] http::Error),

	#[error(display = "Invalid HTTP header value: {}", _0)]
	HttpHeader(#[error(source)] http::header::ToStrError),

	#[error(display = "Netapp error: {}", _0)]
	Netapp(#[error(source)] netapp::error::Error),

	#[error(display = "Sled error: {}", _0)]
	Sled(#[error(source)] sled::Error),

	#[error(display = "Messagepack encode error: {}", _0)]
	RmpEncode(#[error(source)] rmp_serde::encode::Error),
	#[error(display = "Messagepack decode error: {}", _0)]
	RmpDecode(#[error(source)] rmp_serde::decode::Error),
	#[error(display = "JSON error: {}", _0)]
	Json(#[error(source)] serde_json::error::Error),
	#[error(display = "TOML decode error: {}", _0)]
	TomlDecode(#[error(source)] toml::de::Error),

	#[error(display = "Tokio join error: {}", _0)]
	TokioJoin(#[error(source)] tokio::task::JoinError),

	#[error(display = "Remote error: {}", _0)]
	RemoteError(String),

	#[error(display = "Timeout")]
	Timeout,

	#[error(
		display = "Could not reach quorum of {}. {} of {} request succeeded, others returned errors: {:?}",
		_0,
		_1,
		_2,
		_3
	)]
	Quorum(usize, usize, usize, Vec<String>),

	#[error(display = "Bad RPC: {}", _0)]
	BadRpc(String),

	#[error(display = "Corrupt data: does not match hash {:?}", _0)]
	CorruptData(Hash),

	#[error(display = "{}", _0)]
	Message(String),
}

impl From<sled::transaction::TransactionError<Error>> for Error {
	fn from(e: sled::transaction::TransactionError<Error>) -> Error {
		match e {
			sled::transaction::TransactionError::Abort(x) => x,
			sled::transaction::TransactionError::Storage(x) => Error::Sled(x),
		}
	}
}

impl<T> From<tokio::sync::watch::error::SendError<T>> for Error {
	fn from(_e: tokio::sync::watch::error::SendError<T>) -> Error {
		Error::Message("Watch send error".to_string())
	}
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
	fn from(_e: tokio::sync::mpsc::error::SendError<T>) -> Error {
		Error::Message("MPSC send error".to_string())
	}
}

impl<'a> From<&'a str> for Error {
	fn from(v: &'a str) -> Error {
		Error::Message(v.to_string())
	}
}

impl From<String> for Error {
	fn from(v: String) -> Error {
		Error::Message(v)
	}
}

pub trait ErrorContext<T, E> {
	fn err_context<C: std::borrow::Borrow<str>>(self, ctx: C) -> Result<T, Error>;
}

impl<T, E> ErrorContext<T, E> for Result<T, E>
where
	E: std::fmt::Display,
{
	#[inline]
	fn err_context<C: std::borrow::Borrow<str>>(self, ctx: C) -> Result<T, Error> {
		match self {
			Ok(x) => Ok(x),
			Err(e) => Err(Error::Message(format!("{}\n{}", ctx.borrow(), e))),
		}
	}
}

// Custom serialization for our error type, for use in RPC.
// Errors are serialized as a string of their Display representation.
// Upon deserialization, they all become a RemoteError with the
// given representation.

impl Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("{}", self))
	}
}

impl<'de> Deserialize<'de> for Error {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_string(ErrorVisitor)
	}
}

struct ErrorVisitor;

impl<'de> Visitor<'de> for ErrorVisitor {
	type Value = Error;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a string that represents an error value")
	}

	fn visit_str<E>(self, error_msg: &str) -> Result<Self::Value, E> {
		Ok(Error::RemoteError(error_msg.to_string()))
	}

	fn visit_string<E>(self, error_msg: String) -> Result<Self::Value, E> {
		Ok(Error::RemoteError(error_msg))
	}
}
