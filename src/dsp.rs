use fundsp::hacker32::*;

pub fn build_graph() -> Box<dyn AudioUnit> {
    let graph = multipass::<U2>();

    Box::new(graph)
}
