
use diesel::prelude::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::query_builder::*;
use diesel::pg::Pg;
use diesel::sql_types::{BigInt, Text, Date, Timestamp};
use crate::models::{ PaginateError, ErrJson };

use std::str::FromStr;
use gm::utils::pick_datetime_format;

const DEFAULT_PER_PAGE: i64 = 10;



pub trait PaginateCursor: Sized {
    fn paginate_by_cursor(
        self,
        orderField: String,
        sortAscending: Option<bool>,
        // cursor: Option<String>,
        pageBackwards: Option<bool>,
    ) -> PaginatedCursor<Self>;
}

impl<T> PaginateCursor for T {
    fn paginate_by_cursor(
        self,
        orderField: String,
        sortAscending: Option<bool>,
        // cursor: Option<String>,
        pageBackwards: Option<bool>,
    ) -> PaginatedCursor<Self> {

        let ascOrDesc = match sortAscending {
            None => String::from("ASC"),
            Some(a) => match a {
                true => String::from("ASC"),
                false => String::from("DESC")
            }
        };

        PaginatedCursor {
            query: self,
            orderField: orderField,
            count: DEFAULT_PER_PAGE,
            ascendingOrDescending: ascOrDesc,
            cursor: None, // Cannot do dynamic queries yet with QueryFragments
            // cursor_where clause handled in match clause
            pageBackwards: pageBackwards.or(Some(false)).unwrap(),
        }
    }
}


///
#[derive(Debug, Clone, QueryId)]
pub struct PaginatedCursor<T> {
    query: T,
    orderField: String,
    // ConnectionQuery
    count: i64,
    ascendingOrDescending: String, // 'ASC' or 'DESC'
    cursor: Option<String>,
    pageBackwards: bool,
}

impl <T> PaginatedCursor<T> {
    pub fn per_page(self, count: i64) -> Self {
        PaginatedCursor {
            count,
            ..self
        }
    }

    pub fn load_and_count_pages<U>(
        self,
        conn: &PgConnection
    ) -> QueryResult<(Vec<U>, i64, bool)>
        where Self: LoadQuery<PgConnection, (U, i64)>,
            U: std::fmt::Debug
    {

        let count = self.count;
        let pageBackwards = self.pageBackwards;

        // 1. load results
        let mut results = self.load::<(U, i64)>(conn)?;
        // 2. get total pages
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let total_pages = (total as f64 / count as f64).ceil() as i64;
        // overfetched records (self.count + 1) to see if there is a next page
        let is_last_page = count >= (results.len().clone() as i64);

        // 3. Remove overfetched result if not last page
        if !is_last_page {
            let _removed_result = results.pop();
        }

        // 4. remove counts (x.1), leaving just the results (x.0)
        // and collect rows into structs Vec<U>.
        // Flip results if paging backwards
        let records = match pageBackwards {
            true => {
                results.into_iter()
                    .rev()
                    .map(|x| x.0)
                    .collect::<Vec<U>>()
            },
            false => {
                results.into_iter()
                    .map(|x| x.0)
                    .collect::<Vec<U>>()
            }
        };

        Ok((records, total_pages, is_last_page))
    }
}

impl<T: Query> Query for PaginatedCursor<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for PaginatedCursor<T> {}

impl<T> QueryFragment<Pg> for PaginatedCursor<T>
    where T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {

        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        // query is nested in between the brackets (), aliased as q
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") q ");

        /// Issue: can't do dynamic queries. Diesel will build this fragment
        /// once on first query, so cant change between WHERE > and WHERE <
        ///
        /// We can attempt push the entire "WHERE cursor.name > cursor.value"
        /// in as a variable, instead of having a match clause

        // if let Some(cursor) = maybe_cursor {
        //     if page_direction.lessThan {
        //         out.push_sql(" WHERE ");
        //         out.push_identifier(&cursor.name)?;
        //         out.push_sql(" < ");
        //         out.push_bind_param::<Timestamp, _>(&cursor.value)?;
        //     } else {
        //         out.push_sql(" WHERE ");
        //         out.push_identifier(&cursor.name)?;
        //         out.push_sql(" > ");
        //         out.push_bind_param::<Timestamp, _>(&cursor.value)?;
        //     }
        // }

        out.push_sql(" ORDER BY ");
        out.push_identifier(&self.orderField)?;
        out.push_sql(" ");
        out.push_sql(&self.ascendingOrDescending);

        out.push_sql(" LIMIT ");
        // Set count + 1 (overfetch to detect page end)
        out.push_bind_param::<BigInt, _>(&(&self.count + 1))?;
        out.push_sql("; ");

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PageDirection {
    pub lessThan: bool,
    pub queryAscending: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B64Cursor {
    pub name: String,
    pub value: chrono::NaiveDateTime,
}
impl B64Cursor {
    fn new<S: ToString>(name: S, value: S) -> Self {
        Self {
            name: name.to_string(),
            value: chrono::NaiveDateTime::from_str(&value.to_string()).unwrap_or(
                chrono::NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
            ),
        }
    }
    fn to_b64_string(&self) -> String {
        base64::encode(&format!("{}:{}", self.name, self.value))
    }
}

pub fn decode_datetime_cursor(
    encoded_cursor: &str
) -> Result<B64Cursor, PaginateError> {

    let b64decoded_bytes = base64::decode(&encoded_cursor)
        .map_err(PaginateError::from)?;

    let b64decoded = std::str::from_utf8(&b64decoded_bytes)
        .map_err(PaginateError::from)?;

    if !b64decoded.contains(":") {
        return Err(PaginateError::InvalidCursor(ErrJson::new("missing ':' in cursor")))
    }

    // split only the first ':'
    let cursor = b64decoded.clone()
        .splitn(2, ":")
        .collect::<Vec<_>>();

    let cursorName = String::from(*cursor.iter().nth(0).expect("B64Cursor.name missing!"));
    let cursorStr = String::from(*cursor.iter().nth(1).expect("B64Cursor.value missing!"));
    let cursorValue = chrono::NaiveDateTime::parse_from_str(
        &cursorStr,
        pick_datetime_format(&cursorStr),
    );
    // iter() creates references to &str -> &&str. So de-reference with *
    match cursorValue {
        Err(e) => Err(PaginateError::InvalidCursor(errJson!(e))),
        Ok(v) => Ok(B64Cursor {
            name: cursorName,
            value: v,
        })
    }
}


pub fn get_page_direction(
    sortAscending: bool,
    pageBackwards: bool,
) -> PageDirection {
    // Figure out which direction to compare
    // In order to properly page in either direction from a cursor,
    // we sort the query such that the results are always below / after the cursor.
    // Then we later flip the results to match the order we wanted them in.
    match (sortAscending, pageBackwards) {
        (false, false) => PageDirection { lessThan: true,  queryAscending: false },
        (false, true)  => PageDirection { lessThan: false, queryAscending: true },
        (true,  true)  => PageDirection { lessThan: true,  queryAscending: false },
        (true,  false) => PageDirection { lessThan: false, queryAscending: true },
    }
}


