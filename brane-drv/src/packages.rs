use anyhow::Result;
use chrono::{DateTime, Utc};
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use specifications::package::{PackageIndex, PackageInfo};
use uuid::Uuid;

type DateTimeUtc = DateTime<Utc>;

///
///
///
pub async fn get_package_index(graphql_endpoint: &str) -> Result<PackageIndex> {
    #[derive(GraphQLQuery)]
    #[graphql(
        schema_path = "src/graphql/api_schema.json",
        query_path = "src/graphql/get_packages.graphql",
        response_derives = "Debug"
    )]
    pub struct GetPackages;

    let client = Client::new();

    // Prepare GraphQL query.
    let variables = get_packages::Variables {};
    let graphql_query = GetPackages::build_query(variables);

    // Request/response for GraphQL query.
    let graphql_response = client.post(graphql_endpoint).json(&graphql_query).send().await?;
    let graphql_response: Response<get_packages::ResponseData> = graphql_response.json().await?;

    let packages = graphql_response
        .data
        .expect("Expecting zero or more packages.")
        .packages;
    let packages = packages
        .into_iter()
        .map(|p| {
            let functions = p.functions_as_json.map(|f| serde_json::from_str(&f).unwrap());
            let types = p.types_as_json.map(|t| serde_json::from_str(&t).unwrap());

            PackageInfo {
                created: p.created,
                description: p.description.unwrap_or_default(),
                detached: p.detached,
                functions,
                id: p.id,
                kind: p.kind,
                name: p.name,
                owners: p.owners,
                types,
                version: p.version,
            }
        })
        .collect();

    PackageIndex::from_packages(packages)
}
