use axum::{
    http::StatusCode, response::IntoResponse, routing::{get, post}, Extension, Json, Router
};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::{params, Connection, Result};
use aws_config::{imds::client, BehaviorVersion};
use aws_sdk_s3::{config::Region, Client};
use tower::layer;

#[derive(Debug, Serialize, Clone)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let aws_cfg = aws_config::defaults(BehaviorVersion::latest())
        .profile_name("default")
        .region("eu-west-1");

    let sdk_cfg = aws_cfg.load().await;
    let s3_cfg = aws_sdk_s3::config::Builder::from(&sdk_cfg)
    .build();
    let client = Client::from_conf(s3_cfg);

    let conn = Connection::open_in_memory().await?;

    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE person (
                id    INTEGER PRIMARY KEY,
                name  TEXT NOT NULL,
                data  BLOB
            )",
            [],
        )?;

        let steven = Person {
            id: 1,
            name: "Steven".to_string(),
            data: None,
        };

        conn.execute(
            "INSERT INTO person (name, data) VALUES (?1, ?2)",
            params![steven.name, steven.data],
        )?;

        Ok(())
    })
    .await?;

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .layer(Extension(conn))
        .layer(Extension(client));


    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// basic handler that responds with a static string
async fn root(
    Extension(conn): Extension<Connection>,
    Extension(client): Extension<Client>,
) -> impl IntoResponse {

    let people = conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
        let people = stmt
            .query_map([], |row| {
                Ok(Person {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    data: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<Person>, rusqlite::Error>>()?;

        Ok(people)
    })
    .await;

    let _ = download_object(&client).await;

    Json(people.ok()).into_response()
}

pub async fn download_object(
    client: &aws_sdk_s3::Client,
) -> aws_sdk_s3::operation::get_object::GetObjectOutput {
    client
        .get_object()
        .bucket("tvbeat-prod-logs")
        .key("dev/test/test")
        .send()
        .await
        .expect("bla")
}
