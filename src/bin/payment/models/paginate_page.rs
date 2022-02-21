use diesel::prelude::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::query_builder::*;
use diesel::pg::Pg;
use diesel::sql_types::BigInt;


const DEFAULT_PER_PAGE: i64 = 10;


pub trait PaginatePage: Sized {
    fn paginate_by_page(self, page: i64) -> PaginatedPage<Self>;
}

impl<T> PaginatePage for T {
    fn paginate_by_page(self, page: i64) -> PaginatedPage<Self> {
        PaginatedPage {
            query: self,
            count: DEFAULT_PER_PAGE,
            page,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct PaginatedPage<T> {
    query: T,
    page: i64,
    count: i64,
}

impl <T> PaginatedPage<T> {
    pub fn per_page(self, count: i64) -> Self {
        PaginatedPage {
            count,
            ..self
        }
    }

    pub fn load_and_count_pages<U>(
        self,
        conn: &PgConnection
    ) -> QueryResult<(Vec<U>, i64)>
        where Self: LoadQuery<PgConnection, (U, i64)>
    {
        let count = self.count;
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        let total_pages = (total as f64 / count as f64).ceil() as i64;
        Ok((records, total_pages))
    }

}


impl<T: Query> Query for PaginatedPage<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for PaginatedPage<T> {}

impl<T> QueryFragment<Pg> for PaginatedPage<T>
    where T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        // query goes in here, aliased as q
        out.push_sql(") q LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.count)?;
        out.push_sql(" OFFSET ");
        let offset = (self.page - 1) * self.count;
        out.push_bind_param::<BigInt, _>(&offset)?;
        Ok(())
    }
}