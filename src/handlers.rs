use crate::discovery::Discovery;
use log::error;
use warp::{Reply, Rejection};

#[derive(Debug)]
pub struct CloudMapError;
impl warp::reject::Reject for CloudMapError {}

pub async fn cloudmap_sd_handler(discovery: Discovery) -> Result<impl Reply, Rejection> {
    match discovery.discover_targets().await {
        Ok(targets) => Ok(warp::reply::json(&targets)),
        Err(e) => {
            error!("❌ Failed to discover Cloud Map targets: {:?}", e);
            error!("❌ Error details: {}", e);
            Err(warp::reject::custom(CloudMapError))
        }
    }
}
