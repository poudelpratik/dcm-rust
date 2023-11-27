pub mod modules;

use modules::application;

pub async fn init() {
    application::run().await;
}
