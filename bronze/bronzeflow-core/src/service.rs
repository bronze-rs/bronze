pub trait Service {
    fn start(&mut self);

    fn stop(&mut self);
}
