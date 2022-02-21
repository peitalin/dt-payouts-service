

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuery {
    pub count: i64,
    pub sortAscending: Option<bool>,
    pub cursor: Option<String>,
    pub pageBackwards: Option<bool>,
}

#[derive(Debug, Clone, QueryId, Serialize, Deserialize)]
pub struct Connection<T> {
    pub pageInfo: PageInfo,
    pub edges: Vec<Edge<T>>,
    pub totalCount: Option<i64>,
    pub totalAmount: Option<i64>,
    pub totalFees: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub endCursor: Option<String>,
    pub isLastPage: bool,
    pub totalPages: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge<T> {
    pub cursor: Option<String>,
    pub node: T,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBasedConnectionQuery {
    pub pageNumber: i64,
    pub count: i64,
    pub sortAscending: Option<bool>,
}

#[derive(Debug, Clone, QueryId, Serialize, Deserialize)]
pub struct PageBasedConnection<T> {
    pub pageInfo: PageBasedConnectionPageInfo,
    pub edges: Vec<PageBasedEdge<T>>,
    pub totalCount: Option<i64>,
    pub totalAmount: Option<i64>,
    pub totalFees: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBasedConnectionPageInfo {
    pub pageNumber: Option<i64>,
    pub totalPages: Option<i64>,
    pub isLastPage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBasedEdge<T> {
    pub node: T,
}





#[test]
fn deserializes_connection_query_cursor_none() {

    let test_str = r#"
    {
        "sortAscending": false,
        "cursor": null,
        "pageBackwards": false,
        "count": 5
    }
    "#;

    let res = serde_json::from_str::<ConnectionQuery>(test_str);
    match res {
        Ok(cnn) => assert_eq!(cnn.cursor, None),
        Err(e) => panic!(e.to_string()),
    }
}

#[test]
fn deserializes_connection_query_cursor_some() {

    let test_str = r#"
    {
        "sortAscending": false,
        "cursor": "test_cursor",
        "pageBackwards": false,
        "count": 5
    }
    "#;

    let res = serde_json::from_str::<ConnectionQuery>(test_str);
    match res {
        Ok(cnn) => assert_eq!(cnn.cursor, Some(String::from("test_cursor"))),
        Err(e) => panic!(e.to_string()),
    }
}