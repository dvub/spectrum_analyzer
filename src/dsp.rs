use crossbeam_channel::Sender;
use fundsp::hacker32::*;

pub fn build_graph(tx: Sender<f32>) -> Box<dyn AudioUnit> {
    let watcher = join::<U2>()
        >> map(move |i| {
            let _ = tx.try_send(i[0]);
            i[0]
        })
        >> sink();

    let graph = multipass::<U2>() ^ watcher;
    Box::new(graph)
}
