pub mod model;
pub mod render_pipeline;
pub mod renderer;
pub mod wgpu_context;

pub fn load_ron<P, T>(path: P) -> T
where
    P: AsRef<std::path::Path>,
    T: serde::de::DeserializeOwned,
{
    let content = std::fs::read_to_string(path.as_ref()).unwrap_or_else(|e| {
        panic!(
            "the file {} should be able to be read: {}",
            path.as_ref().to_string_lossy(),
            e
        );
    });
    ron::from_str::<T>(&content).unwrap_or_else(|e| {
        panic!("the formatting of the file should be correct: {}", e);
    })
}
pub fn save_ron<P, T>(path: P, value: &T)
where
    P: AsRef<std::path::Path>,
    T: serde::Serialize,
{
    let content = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default()).unwrap();
    std::fs::write(path, content).unwrap();
}
