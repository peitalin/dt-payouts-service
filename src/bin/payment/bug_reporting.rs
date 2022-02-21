extern crate bugsnag;

use std::panic;
use std::sync::{Arc, Mutex};

/// The wrapper for storing the global instance
struct GlobalInstanceWrapper {
    instance: Option<Arc<Mutex<bugsnag::Bugsnag>>>,
}

impl GlobalInstanceWrapper {
    pub fn new() -> GlobalInstanceWrapper {
        GlobalInstanceWrapper { instance: None }
    }

    pub fn instance(&mut self) -> Option<Arc<Mutex<bugsnag::Bugsnag>>> {
        self.instance.clone()
    }

    pub fn set_instance(&mut self, instance: bugsnag::Bugsnag) {
        self.instance = Some(Arc::new(Mutex::new(instance)))
    }
}

// The global instance that holds our wrapper
lazy_static! {
    static ref INSTANCE: Mutex<GlobalInstanceWrapper> = Mutex::new(GlobalInstanceWrapper::new());
}

/// Returns the global api object
/// To be accessible by this function, the api object needs to be registered
/// with the 'to_global_instance' function!
pub fn global_instance() -> Option<Arc<Mutex<bugsnag::Bugsnag>>> {
    match INSTANCE.lock() {
        Ok(mut res) => res.instance(),
        Err(_) => None,
    }
}

/// Consumes the api object and registers this object as the global api object
pub fn to_global_instance(api: bugsnag::Bugsnag) {
    if let Ok(mut res) = INSTANCE.lock() {
        res.set_instance(api);
    }
}

/// Converts our api object to the global api object and registers the panic
/// handler. This panic handler will use the global api object, if called.
fn register_panic_handler_with_global_instance(api: bugsnag::Bugsnag) {
    to_global_instance(api);

    panic::set_hook(Box::new(|info| {
        if let Some(api_mtx) = global_instance() {
            if let Ok(api) = api_mtx.lock() {
                if bugsnag::panic::handle(
                    &api,
                    &info,
                    Some(&["register_panic_handler_with_global_instance"]),
                ).is_err()
                {
                    println!("Error at notifying bugsnag!");
                }
            }
        }
    }));
}

pub fn setup_bug_reporting() {
    // Set up bagsnag if applicable
    if let Some(api_key) = get_bugsnag_api_key() {
      let mut bugsnagApi = bugsnag::Bugsnag::new(&api_key, env!("CARGO_MANIFEST_DIR"));
      bugsnagApi.set_app_info(Some(env!("CARGO_PKG_VERSION")), Some(&get_gm_environment()), Some("rust"));
      register_panic_handler_with_global_instance(bugsnagApi);
    }
}

fn get_bugsnag_api_key() -> Option<String> {
    dotenv::dotenv().ok();
    std::env::var("BUGSNAG_KEY").ok()
}

pub fn get_gm_environment() -> String {
  dotenv::dotenv().ok();
  std::env::var("gm_ENV")
      .expect("gm_ENV missing in .env")
}