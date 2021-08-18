mod get_profile;
use crate::metrics::{
    start_request, {end_request, LabelledReply, Metrics},
};
use crate::models::in_memory_database::InMemoryDatabase;
use std::{convert::Infallible, sync::Arc};
use warp::{hyper::StatusCode, wrap_fn, Filter, Rejection, Reply};

pub fn handle_all_routes(
    memory_database: Arc<InMemoryDatabase>,
    metrics: Arc<Metrics>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let get_profile = get_profile::get_profile(memory_database.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS", "PUT", "PATCH"])
        .allow_headers(vec!["Origin", "Content-Type", "X-Auth-Token", "X-AppId"]);
    let routes_with_labels = warp::path!("api" / "v1" / ..)
        .and(get_profile.map(|reply| LabelledReply::new(reply, "get_profile")));
    routes_with_labels
        .with(wrap_fn(|f| wrap_metrics(f, metrics.clone())))
        .recover(handle_rejection)
        .with(cors)
}
// We turn Rejection into Reply to workaround warp not setting CORS headers on rejections.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status(
        format!("{:?}", err),
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}
fn wrap_metrics<F>(
    filter: F,
    metrics: Arc<Metrics>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone
where
    F: Filter<Extract = (LabelledReply,), Error = Rejection> + Clone + Send + Sync + 'static,
{
    warp::any()
        .and(start_request())
        .and(filter)
        .map(move |timer, reply| end_request(metrics.clone(), timer, reply))
}
