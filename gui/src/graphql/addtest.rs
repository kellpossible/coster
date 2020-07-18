use super::client::Client;
use futures::future::FutureExt;
use graphql_client::GraphQLQuery;
use log::error;
use wasm_bindgen_futures::spawn_local;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/addtest.graphql",
    response_derives = "Debug"
)]
pub struct AddTest;

pub fn add_test() {
    let client = Client::new("http://localhost:8000/api");

    let variables = add_test::Variables { a: 20, b: 30 };

    let response = client.call(AddTest, variables);

    let response = response.map(|response| {
        if let Err(err) = response {
            error!("Cannot perform AddTest query: {:?}", err);
        }
    });

    spawn_local(response);
}
