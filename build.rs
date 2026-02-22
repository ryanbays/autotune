fn main() {
    let config = rust_to_mermaid::build_diagram::DiagramConfig {
        main_title: "Autotune",
        ..Default::default()
    };
    let _ = rust_to_mermaid::build_diagram::generate_diagrams_with_config(&config);
}
