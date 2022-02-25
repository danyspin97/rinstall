#[derive(Parser)]
pub struct GenerateRpmFiles {
    #[clap(from_global)]
    pub packages: Vec<String>,
}
