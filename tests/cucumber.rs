use cucumber::World;

#[derive(Debug, Default, World)]
pub struct GnoemWorld;

fn main() {
    futures::executor::block_on(GnoemWorld::run("tests/features"));
}
