use mongodb::options::ClientOptions;
use tokio::test;
use teo::connectors::mongodb::MongoDBConnectorHelpers;
use teo::core::graph::Graph;


async fn make_graph() -> &'static Graph {

    let options = ClientOptions::parse("mongodb://localhost:27017/teotestserver").await.unwrap();

    let graph = Box::leak(Box::new(Graph::new(|g| {
        g.mongodb(options.clone());

        g.reset_database();

        g.model("Required", |m| {
            m.field("string", |f| {
                f.required().string();
            });
        });
    }).await));

    graph
}

#[test]
async fn server_should_start() -> std::io::Result<()> {
    let graph = make_graph().await;
    graph.start_server(5000)
}
