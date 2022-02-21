use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;


/// Implemented by types which represent stripe objects.
pub trait Object {
    /// The canonical id type for this object.
    type Id;
    /// The id of the object.
    fn id(&self) -> Self::Id;
    /// The object's type, typically represented in wire format as the `object` property.
    fn object(&self) -> &'static str;
}

/// A deleted object.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deleted<T> {
    /// Unique identifier for the object.
    pub id: T,
    /// Unique identifier for the object.
    pub object: String,
    /// Always true for a deleted object.
    pub deleted: bool,
}

/// The `Expand` struct is used to serialize `expand` arguments in retrieve and list apis.
#[derive(Serialize)]
pub struct Expand {
    #[serde(skip_serializing_if = "Expand::is_empty")]
    pub expand: Vec<String>,
}

impl Expand {
    pub(crate) fn is_empty(expand: &Vec<String>) -> bool {
        expand.is_empty()
    }
}

/// An id or object.
///
/// By default stripe will return an id for most fields, but if more detail is
/// necessary the `expand` parameter can be provided to ask for the id to be
/// loaded as an object instead:
///
/// ```rust,ignore
/// Charge::retrieve(&client, &charge_id, &["invoice.customer"])
/// ```
///
/// See [https://stripe.com/docs/api/expanding_objects](https://stripe.com/docs/api/expanding_objects).
#[derive(Clone, Debug, Serialize, Deserialize)] // TODO: Implement deserialize by hand for better error messages
#[serde(untagged)]
pub enum Expandable<T: Object> {
    Id(T::Id),
    Object(Box<T>),
}

impl<T> Expandable<T>
where
    T: Object,
    T::Id: Clone,
{
    pub fn id(&self) -> T::Id {
        match self {
            Expandable::Id(id) => id.clone(),
            Expandable::Object(obj) => obj.id(),
        }
    }
}

impl<T: Object> Expandable<T> {
    pub fn is_object(&self) -> bool {
        match self {
            Expandable::Id(_) => false,
            Expandable::Object(_) => true,
        }
    }

    pub fn as_object(&self) -> Option<&T> {
        match self {
            Expandable::Id(_) => None,
            Expandable::Object(obj) => Some(&obj),
        }
    }

    pub fn to_object(self) -> Option<T> {
        match self {
            Expandable::Id(_) => None,
            Expandable::Object(obj) => Some(*obj),
        }
    }
}

/// Implemented by types which support cursor-based pagination,
/// typically with an id, allowing them to be fetched using a `List`
/// returned by the corresponding "list" api request.
pub trait Paginate {
    type Cursor: AsCursor;
    fn cursor(&self) -> Self::Cursor;
}

pub trait AsCursor: AsRef<str> {}

impl<'a> AsCursor for &'a str {}
impl AsCursor for String {}

impl<T> Paginate for T
where
    T: Object,
    T::Id: AsCursor,
{
    type Cursor = T::Id;
    fn cursor(&self) -> Self::Cursor {
        self.id()
    }
}

/// A single page of a cursor-paginated list of an object.
#[derive(Debug, Deserialize, Serialize)]
pub struct List<T> {
    pub object: String,
    pub data: Vec<T>,
    pub has_more: bool,
    pub total_count: Option<u64>,
    pub url: String,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List {
            object: String::new(),
            data: Vec::new(),
            has_more: false,
            total_count: None,
            url: String::new()
        }
    }
}

impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        List {
            object: self.object.clone(),
            data: self.data.clone(),
            has_more: self.has_more,
            total_count: self.total_count,
            url: self.url.clone(),
        }
    }
}

// impl<T: DeserializeOwned + Send + 'static> List<T> {
//     /// Prefer `List::next` when possible
//     pub fn get_next(client: &StripeClient, url: &str, last_id: &str) -> Response<List<T>> {
//         if url.starts_with("/v1/") {
//             // TODO: Maybe parse the URL?  Perhaps `List` should always parse its `url` field.
//             let mut url = url.trim_start_matches("/v1/").to_string();
//             if url.contains('?') {
//                 url.push_str(&format!("&starting_after={}", last_id));
//             } else {
//                 url.push_str(&format!("?starting_after={}", last_id));
//             }
//             async move {
//                 client.get(&url).await
//             }
//         } else {
//             err(Error::Unsupported("URL for fetching additional data uses different API version"))
//         }
//     }
// }

// impl<T: Paginate + DeserializeOwned + Send + 'static> List<T> {
//     /// Repeatedly queries Stripe for more data until all elements in list are fetched, using
//     /// Stripe's default page size.
//     ///
//     /// Not supported by `stripe::async::Client`.
//     #[cfg(not(feature = "async"))]
//     pub fn get_all(self, client: &StripeClient) -> Response<Vec<T>> {
//         let mut data = Vec::new();
//         let mut next = self;
//         loop {
//             if next.has_more {
//                 let resp = next.next(&client)?;
//                 data.extend(next.data);
//                 next = resp;
//             } else {
//                 data.extend(next.data);
//                 break;
//             }
//         }
//         Ok(data)
//     }

//     /// Fetch an additional page of data from stripe.
//     pub fn next(&self, client: &StripeClient) -> Response<List<T>> {
//         if let Some(last_id) = self.data.last().map(|d| d.cursor()) {
//             List::get_next(client, &self.url, last_id.as_ref())
//         } else {
//             ok(List {
//                 object: self.object.clone(),
//                 data: Vec::new(),
//                 has_more: false,
//                 total_count: self.total_count,
//                 url: self.url.clone(),
//             })
//         }
//     }
// }

pub type Metadata = HashMap<String, String>;
pub type Timestamp = i64;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct RangeBounds<T> {
    pub gt: Option<T>,
    pub gte: Option<T>,
    pub lt: Option<T>,
    pub lte: Option<T>,
}

impl<T> Default for RangeBounds<T> {
    fn default() -> Self {
        RangeBounds { gt: None, gte: None, lt: None, lte: None }
    }
}

/// A set of generic request parameters that can be used on
/// list endpoints to filter their results by some timestamp.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RangeQuery<T> {
    Exact(T),
    Bounds(RangeBounds<T>),
}

impl<T> RangeQuery<T> {
    /// Filter results to exactly match a given value
    pub fn eq(value: T) -> RangeQuery<T> {
        RangeQuery::Exact(value)
    }

    /// Filter results to be after a given value
    pub fn gt(value: T) -> RangeQuery<T> {
        let mut bounds = RangeBounds::default();
        bounds.gt = Some(value);
        RangeQuery::Bounds(bounds)
    }

    /// Filter results to be after or equal to a given value
    pub fn gte(value: T) -> RangeQuery<T> {
        let mut bounds = RangeBounds::default();
        bounds.gte = Some(value);
        RangeQuery::Bounds(bounds)
    }

    /// Filter results to be before to a given value
    pub fn lt(value: T) -> RangeQuery<T> {
        let mut bounds = RangeBounds::default();
        bounds.gt = Some(value);
        RangeQuery::Bounds(bounds)
    }

    /// Filter results to be before or equal to a given value
    pub fn lte(value: T) -> RangeQuery<T> {
        let mut bounds = RangeBounds::default();
        bounds.gte = Some(value);
        RangeQuery::Bounds(bounds)
    }
}
