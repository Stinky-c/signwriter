use url::Url;

#[derive(Default)]
pub struct Service {
    pub service_name: String,
    // pub urls: Vec<Url>,
    pub urls: Vec<String>,
}
