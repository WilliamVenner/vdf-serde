//! Support for the [Valve Data Format](https://developer.valvesoftware.com/wiki/KeyValues) for [Serde](https://serde.rs/).
//!
//! Based on the [steamy-vdf](https://crates.io/crates/steamy-vdf) VDF parser library.
//!
//! # Simple Example
//!
//! ```
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Example {
//!     thing: String,
//!     other_thing: bool,
//!     more_stuff: Inner,
//!     some_opt: Option<String>,
//! }
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Inner {
//!     bonus_content: u8,
//!     coolness: f64,
//! }
//!
//! let vdf_data = "\"Example\"
//! {
//! \t\"thing\"\t\"hello\"
//! \t\"other_thing\"\t\"1\"
//! \t\"more_stuff\"
//! \t{
//! \t\t\"bonus_content\"\t\"69\"
//! \t\t\"coolness\"\t\"420.1337\"
//! \t}
//! \t\"some_opt\"\t\"hello\"
//! }";
//! let data = Example {
//!     thing: "hello".to_string(),
//!     other_thing: true,
//!     some_opt: Some("hello".to_string()),
//!     more_stuff: Inner {
//!         bonus_content: 69,
//!         coolness: 420.1337,
//!     },
//! };
//!
//! assert_eq!(vdf_serde::to_string(&data)?, vdf_data);
//! assert!(vdf_serde::from_str::<Example>(vdf_data).is_ok());
//! # Ok::<(), vdf_serde::Error>(())
//! ```
//!
//! # Example I Needed When I Wrote This Library
//!
//! If you've got a pile of arbitrary key-value data you can't predict the structure of
//! in advance, you can use a `HashMap<K, V>`. But if there's a top-level name attached,
//! you'll need a newtype, like this:
//!
//! ```
//! use std::collections::HashMap;
//! use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct LibraryFolders(HashMap<String, String>);
//!
//! let vdf_data = r#""LibraryFolders"
//! {
//!     "TimeNextStatsReport"   "69420691337"
//!     "ContentStatsID"        "31337131269420"
//!     "1"                     "D:\\SteamLibrary"
//! }"#;
//! let LibraryFolders(data) = vdf_serde::from_str(vdf_data)?;
//! let expected = [("TimeNextStatsReport", "69420691337"), ("ContentStatsID", "31337131269420"), ("1", r"D:\SteamLibrary")]
//!     .iter()
//!     .map(|(a, b)| (a.to_string(), b.to_string()))
//!     .collect::<HashMap<_, _>>();
//! assert_eq!(data, expected);
//! # Ok::<(), vdf_serde::Error>(())
//! ```
//!
//! # Notes
//!
//! The VDF format is rather drastically underspecified, so until I figure out a way to implement them in a way that's compatible with
//! existing VDF files, the following types from [the Serde data model](https://serde.rs/data-model.html) are unsupported:
//!
//! - byte array
//! - option
//! - unit `()`
//! - unit_struct `struct WillNotWork;`
//! - newtype_variant `enum Broken { Example(u8) }`
//! - seq `Vec<T>`
//! - tuple
//! - tuple_struct `struct Unsupported(u8, bool, char);`
//! - tuple_variant `enum Bad { NotWorking(u8, bool, char) }`
//! - struct_variant `enum Nope { NotHappening { datum: u8 } }`
//!
//! You might wind up needing to [implement Serialize yourself](https://serde.rs/impl-serialize.html) and
//! [implement Deserialize yourself](https://serde.rs/impl-deserialize.html) if you use anything like this.
//! The rest of the Serde data model works, though, although maps with non-atomic keys might be a bit of a mess.
//!
//! ```
//! use std::collections::HashMap as Map;
//! use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! enum UnitVariants { A, B }
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct NewtypeStruct(u8);
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct KitchenSink {
//!     a: bool,
//!     b: i8,
//!     c: i16,
//!     d: i32,
//!     e: i64,
//!     f: u8,
//!     g: u16,
//!     h: u32,
//!     i: u64,
//!     j: f32,
//!     k: f64,
//!     l: char,
//!     m: String,
//!     n: UnitVariants,
//!     o: NewtypeStruct,
//!     p: Map<String, String>,
//! }
//!
//! # let mut p = Map::new();
//! # p.insert("hello".to_string(), "there".to_string());
//! let data = KitchenSink { // yada yada yada
//! # a: false, b: -123, c: -45, d: -67, e: -890, f: 12, g: 345, h: 678, i: 901, j: 0.3,
//! # k: 9.25, l: '???', m: "sample text".to_string(), n: UnitVariants::A,
//! # o: NewtypeStruct(3), p,
//! };
//! assert_eq!(data, vdf_serde::from_str(&vdf_serde::to_string(&data)?)?);
//! # Ok::<(), vdf_serde::Error>(())
//! ```
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/vdf-serde/0.3.0")]

mod de;
mod error;
mod ser;

pub use de::{from_str, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_string, Serializer};
