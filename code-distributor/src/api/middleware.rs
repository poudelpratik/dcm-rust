use crate::connection_handler::WarpError;
use crate::AppData;
use http::HeaderMap;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::Filter;

async fn validate_api_key(
    app_data: Arc<AppData>,
    headers: HeaderMap,
) -> Result<(), warp::Rejection> {
    let api_key = headers
        .get("X-Api-Key")
        .ok_or_else(|| warp::reject::custom(WarpError))?
        .to_str()
        .map_err(|_| warp::reject::custom(WarpError))?;
    if api_key == app_data.config.api_key {
        Ok(())
    } else {
        Err(warp::reject::custom(WarpError))
    }
}

pub fn api_key_filter(
    app_data_filter: BoxedFilter<(Arc<AppData>,)>,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    app_data_filter
        .and(warp::header::headers_cloned())
        .and_then(validate_api_key)
        .untuple_one()
}
