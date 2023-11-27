use crate::configuration::Configuration;
use crate::fragment_registry::fragment::Fragment;
use crate::fragment_registry::FragmentRegistry;
use client_registry::ClientRegistry;
use std::path::PathBuf;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod api;
pub mod client_registry;
pub mod configuration;
pub mod connection_handler;
pub mod fragment_executor;
pub mod fragment_registry;
pub mod util;

static INIT_CALLED: AtomicBool = AtomicBool::new(false);

pub async fn init(config: Configuration) {
    // set env var enable log
    if INIT_CALLED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        panic!("Initialization already occurred");
    }

    // Initialize the fragment registry
    // Parse executable_fragments.json file and store it in the fragment registry
    let final_fragments_json =
        util::file_handler::read(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fragments").join("executable_fragments.json"))
            .expect("Unable to read executable_fragments.json file");
    let fragments = serde_json::from_str::<Vec<Fragment>>(final_fragments_json.as_str())
        .expect("Unable to parse executable_fragments.json file");
    let fragment_registry = FragmentRegistry::new(fragments);

    // Initialize the client registry
    let client_registry = Arc::new(Mutex::new(ClientRegistry::new()));

    // Initialize the ApplicationContext
    let app_context = Arc::new(Mutex::new(ApplicationContext {
        config: Arc::new(config),
        client_registry,
        fragment_registry,
    }));

    connection_handler::initialize(app_context.clone()).await;
}

pub(crate) struct ApplicationContext {
    pub config: Arc<Configuration>,
    pub client_registry: Arc<Mutex<ClientRegistry>>,
    pub fragment_registry: FragmentRegistry,
}
